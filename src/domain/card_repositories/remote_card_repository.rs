use anyhow::{anyhow, Context, Result};
use chrono::{DateTime, Local};
use quick_xml::de as xml;
use reqwest::{blocking::Client, Method};
use serde::Deserialize;

use crate::domain::{Card, CardRepository};

pub struct RemoteCardRepository<'a> {
    pub addressbook_path: String,
    pub client: &'a Client,
}

impl<'a> RemoteCardRepository<'a> {
    pub fn new(host: &str, client: &'a Client) -> Result<Self> {
        Ok(Self {
            addressbook_path: format!("{}{}", host, addressbook_path(host, client)?),
            client,
        })
    }
}

impl<'a> CardRepository for RemoteCardRepository<'a> {
    fn create(&self, card: &mut Card) -> Result<()> {
        let res = self
            .client
            .put(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .body(card.raw.clone())
            .send()
            .with_context(|| "cannot create card")?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(anyhow!(reason).context("cannot create card"));
        }

        card.etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .or_else(|| card.etag.as_deref())
            .map(String::from);

        Ok(())
    }

    fn read(&self, id: &str) -> Result<Card> {
        let res = self
            .client
            .get(format!("{}{}.vcf", self.addressbook_path, id))
            .basic_auth("user", Some(""))
            .header("Depth", "1")
            .send()
            .with_context(|| anyhow!(r#"cannot read card "{}""#, id))?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(anyhow!(reason).context(format!(r#"cannot read card "{}""#, id)));
        }

        let date = res
            .headers()
            .get("last-modified")
            .ok_or_else(|| anyhow!(r#"cannot get last modified date of card "{}""#, id))?;
        let date = date
            .to_str()
            .with_context(|| anyhow!(r#"cannot parse last modified date of card "{}""#, id))?;
        let date = DateTime::parse_from_rfc2822(date)
            .with_context(|| anyhow!(r#"cannot parse last modified date of card "{}""#, id))?
            .with_timezone(&Local);
        let etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .map(String::from);
        let raw = res
            .text()
            .context(anyhow!(r#"cannot read content of card "{}""#, id))?;

        Ok(Card {
            id: id.to_owned(),
            etag,
            date,
            raw,
        })
    }

    fn read_all(&self) -> Result<Vec<Card>> {
        todo!()
    }

    fn update(&self, card: &mut Card) -> Result<()> {
        let mut req = self
            .client
            .put(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""))
            .header("Content-Type", "text/vcard; charset=utf-8")
            .body(card.raw.clone());

        if let Some(etag) = card.etag.as_deref() {
            req = req.header("If-Match", etag);
        }

        let res = req
            .send()
            .with_context(|| format!(r#"cannot update card "{}""#, card.id))?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(anyhow!(reason).context(format!(r#"cannot update card "{}""#, card.id)));
        }

        card.etag = res
            .headers()
            .get("etag")
            .and_then(|h| h.to_str().ok())
            .or_else(|| card.etag.as_deref())
            .map(String::from);

        Ok(())
    }

    fn delete(&self, card: &Card) -> Result<()> {
        let mut req = self
            .client
            .delete(format!("{}{}.vcf", self.addressbook_path, card.id))
            .basic_auth("user", Some(""));

        if let Some(etag) = card.etag.as_deref() {
            req = req.header("If-Match", etag);
        }

        let res = req
            .send()
            .with_context(|| format!(r#"cannot delete card "{}""#, card.id))?;
        let res_status = res.status();

        if !res_status.is_success() {
            let reason = res.text().unwrap_or(res_status.to_string());
            return Err(anyhow!(reason).context(format!(r#"cannot delete card "{}""#, card.id)));
        }

        Ok(())
    }
}

// Common structs

#[derive(Debug, Deserialize)]
pub struct Multistatus<T> {
    #[serde(rename = "response")]
    pub responses: Vec<Response<T>>,
}

#[derive(Debug, Deserialize)]
pub struct Response<T> {
    pub href: Href,
    pub propstat: Propstat<T>,
}

#[derive(Debug, Deserialize)]
pub struct Propstat<T> {
    pub prop: T,
    pub status: Option<Status>,
}

#[derive(Debug, Deserialize)]
pub struct Href {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct Status {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct GetCtag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct GetEtag {
    #[serde(rename = "$value")]
    pub value: String,
}

#[derive(Debug, Deserialize)]
pub struct GetLastModified {
    #[serde(with = "date_parser", rename = "$value")]
    pub value: DateTime<Local>,
}

mod date_parser {
    use chrono::{DateTime, Local};
    use serde::{self, Deserialize, Deserializer};

    pub fn deserialize<'de, D>(deserializer: D) -> Result<DateTime<Local>, D::Error>
    where
        D: Deserializer<'de>,
    {
        let s = String::deserialize(deserializer)?;
        DateTime::parse_from_rfc2822(&s)
            .map(|d| d.into())
            .map_err(serde::de::Error::custom)
    }
}

// Current user principal structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct CurrentUserPrincipalProp {
    pub current_user_principal: CurrentUserPrincipal,
}

#[derive(Debug, Deserialize)]
struct CurrentUserPrincipal {
    pub href: Href,
}

// Addressbook home set structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookHomeSetProp {
    pub addressbook_home_set: AddressbookHomeSet,
}

#[derive(Debug, Deserialize)]
struct AddressbookHomeSet {
    pub href: Href,
}

// Addressbook structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
struct AddressbookProp {
    pub resourcetype: AddressbookResourceType,
}

#[derive(Debug, Deserialize)]
struct AddressbookResourceType {
    pub addressbook: Option<Addressbook>,
}

#[derive(Debug, Deserialize)]
struct Addressbook {}

// Address data structs

#[derive(Debug, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub struct AddressDataProp {
    pub address_data: AddressData,
    pub getetag: GetEtag,
    pub getlastmodified: GetLastModified,
}

#[derive(Debug, Deserialize)]
pub struct AddressData {
    #[serde(rename = "$value")]
    pub value: String,
}

// Ctag structs

#[derive(Debug, Deserialize)]
pub struct CtagProp {
    pub getctag: GetCtag,
}

// Methods

fn propfind() -> Result<Method> {
    Method::from_bytes(b"PROPFIND").context(r#"cannot create custom method "PROPFIND""#)
}

fn fetch_current_user_principal_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, format!("{}{}", host, path))
        .basic_auth("user", Some(""))
        .body(
            r#"
            <D:propfind xmlns:D="DAV:">
                <D:prop>
                    <D:current-user-principal />
                </D:prop>
            </D:propfind>
            "#,
        )
        .send()
        .context("cannot send current user principal request")?;
    let res = res
        .text()
        .context("cannot extract text body from current user principal response")?;
    let res: Multistatus<CurrentUserPrincipalProp> =
        xml::from_str(&res).context("cannot parse current user principal response")?;

    Ok(res
        .responses
        .first()
        .map(|res| {
            res.propstat
                .prop
                .current_user_principal
                .href
                .value
                .to_owned()
        })
        .unwrap_or(path))
}

fn fetch_addressbook_home_set_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, format!("{}{}", host, path))
        .basic_auth("user", Some(""))
        .body(
            r#"
            <D:propfind xmlns:D="DAV:" xmlns:C="urn:ietf:params:xml:ns:carddav">
                <D:prop>
                    <C:addressbook-home-set />
                </D:prop>
            </D:propfind>
            "#,
        )
        .send()
        .context("cannot send addressbook home set request")?;
    let res = res
        .text()
        .context("cannot extract text body from addressbook home set response")?;
    let res: Multistatus<AddressbookHomeSetProp> =
        xml::from_str(&res).context("cannot parse addressbook home set response")?;

    Ok(res
        .responses
        .first()
        .map(|res| res.propstat.prop.addressbook_home_set.href.value.to_owned())
        .unwrap_or(path))
}

fn fetch_addressbook_url(host: &str, path: String, client: &Client) -> Result<String> {
    let res = client
        .request(propfind()?, host)
        .basic_auth("user", Some(""))
        .send()
        .context("cannot send addressbook request")?;
    let res = res
        .text()
        .context("cannot extract text body from addressbook response")?;
    let res: Multistatus<AddressbookProp> =
        xml::from_str(&res).context("cannot parse addressbook response")?;

    Ok(res
        .responses
        .iter()
        .find(|res| {
            let valid_status = res
                .propstat
                .status
                .as_ref()
                .map(|s| s.value.ends_with("200 OK"))
                .unwrap_or(false);
            let has_addressbook = res
                .propstat
                .prop
                .resourcetype
                .addressbook
                .as_ref()
                .is_some();

            valid_status && has_addressbook
        })
        .map(|res| res.href.value.to_owned())
        .unwrap_or(path))
}

pub fn addressbook_path(host: &str, client: &Client) -> Result<String> {
    let path = String::from("/");
    let path = fetch_current_user_principal_url(host, path, client)?;
    let path = fetch_addressbook_home_set_url(host, path, client)?;
    let path = fetch_addressbook_url(host, path, client)?;
    Ok(path)
}
