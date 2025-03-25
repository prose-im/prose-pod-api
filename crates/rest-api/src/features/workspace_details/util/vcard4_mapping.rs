// prose-pod-api
//
// Copyright: 2025, Rémi Bardon <remi@remibardon.name>
// License: Mozilla Public License v2.0 (MPL v2.0)

use std::str::FromStr as _;

use minidom::Element;
use service::prose_xmpp::{
    ns,
    stanza::{vcard4 as prose_xmpp_properties, VCard4},
};
use vcard4::property::{self as vcard4_properties, ExtensionProperty};

pub fn prose_xmpp_vcard4_to_vcard4_vcard(
    VCard4 {
        adr,
        email,
        fn_,
        n,
        impp,
        nickname,
        note,
        org,
        role,
        tel,
        title,
        url,
        unknown_properties,
    }: VCard4,
) -> Result<vcard4::Vcard, vcard4::Error> {
    let mut formatted_names = fn_.into_iter();
    let Some(formatted_name) = formatted_names.next() else {
        return Err(vcard4::Error::NoFormattedName);
    };
    let mut builder = vcard4::VcardBuilder::new(formatted_name.value);
    for formatted_name in formatted_names {
        builder = builder.formatted_name(formatted_name.value);
    }

    for adr in adr {
        builder = builder.address(prose_xmpp_adr_to_vcard4_delivery_address(adr));
    }
    for email in email {
        builder = builder.email(email.value);
    }
    for name in n {
        builder = builder.name(prose_xmpp_name_to_vcard4_name(name)?);
    }
    for impp in impp {
        builder = builder.impp(vcard4::Uri::from_str(&impp.value)?);
    }
    for nickname in nickname {
        builder = builder.nickname(nickname.value);
    }
    for note in note {
        builder = builder.note(note.value);
    }
    for org in org {
        builder = builder.org(vec![org.value]);
    }
    for role in role {
        builder = builder.role(role.value);
    }
    for tel in tel {
        builder = builder.telephone(tel.value);
    }
    for title in title {
        builder = builder.title(title.value);
    }
    for url in url {
        builder = builder.url(vcard4::Uri::from_str(&url.value)?);
    }

    let mut vcard = builder.finish();
    for element in unknown_properties.into_iter() {
        if !element.name().to_ascii_lowercase().starts_with("x-") {
            continue;
        }
        vcard.extensions.push(ExtensionProperty {
            name: element.name().to_ascii_uppercase(),
            // NOTE: This is opinionated.
            value: vcard4_properties::AnyProperty::Text(element.text()),
            // NOTE: This too.
            group: Default::default(),
            // NOTE: And this too.
            parameters: Default::default(),
        });
    }

    Ok(vcard)
}

pub fn vcard4_vcard_to_prose_xmpp_vcard4(
    vcard4::Vcard {
        formatted_name,
        name,
        nickname,
        url,
        address,
        tel,
        email,
        impp,
        title,
        role,
        org,
        note,
        extensions,
        ..
    }: &vcard4::Vcard,
) -> Result<VCard4, vcard4::Error> {
    Ok(VCard4 {
        adr: address
            .iter()
            .map(|p| &p.value)
            .map(vcard4_delivery_address_to_prose_xmpp_adr)
            .collect(),
        email: email
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Email { value: s.clone() })
            .collect(),
        fn_: formatted_name
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Fn_ { value: s.clone() })
            .collect(),
        n: name
            .iter()
            .map(|p| &p.value)
            .map(vcard4_name_to_prose_xmpp_name)
            .collect(),
        impp: impp
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Impp {
                value: s.to_string(),
            })
            .collect(),
        nickname: nickname
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Nickname { value: s.clone() })
            .collect(),
        note: note
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Note { value: s.clone() })
            .collect(),
        org: org
            .iter()
            .map(|p| &p.value)
            .map(|v| v.first())
            .flatten()
            .map(|s| prose_xmpp_properties::Org { value: s.clone() })
            .collect(),
        role: role
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Role { value: s.clone() })
            .collect(),
        tel: tel
            .iter()
            .map(|p| match p {
                vcard4_properties::TextOrUriProperty::Text(p) => p.value.clone(),
                vcard4_properties::TextOrUriProperty::Uri(p) => p.value.to_string(),
            })
            .map(|s| prose_xmpp_properties::Tel { value: s.clone() })
            .collect(),
        title: title
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::Title { value: s.clone() })
            .collect(),
        url: url
            .iter()
            .map(|p| &p.value)
            .map(|s| prose_xmpp_properties::URL {
                value: s.to_string(),
            })
            .collect(),
        unknown_properties: extensions
            .iter()
            .map(|p| {
                Element::builder(p.name.to_ascii_lowercase(), ns::VCARD4)
                    // NOTE: This is opinionated.
                    .append(p.value.to_string())
                    .build()
            })
            .collect(),
    })
}

pub fn prose_xmpp_adr_to_vcard4_delivery_address(
    prose_xmpp_properties::Adr {
        code,
        country,
        ext,
        locality,
        pobox,
        region,
        street,
    }: prose_xmpp_properties::Adr,
) -> vcard4_properties::DeliveryAddress {
    vcard4_properties::DeliveryAddress {
        po_box: pobox.first().cloned(),
        extended_address: ext.first().cloned(),
        street_address: street.first().cloned(),
        locality: locality.first().cloned(),
        region: region.first().cloned(),
        postal_code: code.first().cloned(),
        country_name: country.first().cloned(),
    }
}

pub fn vcard4_delivery_address_to_prose_xmpp_adr(
    vcard4_properties::DeliveryAddress {
        po_box,
        extended_address,
        street_address,
        locality,
        region,
        postal_code,
        country_name,
    }: &vcard4_properties::DeliveryAddress,
) -> prose_xmpp_properties::Adr {
    prose_xmpp_properties::Adr {
        code: postal_code.clone().into_iter().collect(),
        country: country_name.clone().into_iter().collect(),
        ext: extended_address.clone().into_iter().collect(),
        locality: locality.clone().into_iter().collect(),
        pobox: po_box.clone().into_iter().collect(),
        region: region.clone().into_iter().collect(),
        street: street_address.clone().into_iter().collect(),
    }
}

/// `[family name, given name, additional names, honorific prefixes, honorific suffixes]`.
pub fn prose_xmpp_name_to_vcard4_name(
    prose_xmpp_properties::Name {
        surname,
        given,
        additional,
    }: prose_xmpp_properties::Name,
) -> Result<[String; 5], vcard4::Error> {
    let Some(surname) = surname else {
        return Err(vcard4::Error::InvalidPropertyValue);
    };
    let Some(given) = given else {
        return Err(vcard4::Error::InvalidPropertyValue);
    };
    let Some(additional) = additional else {
        return Err(vcard4::Error::InvalidPropertyValue);
    };
    Ok([
        surname,
        given,
        additional,
        String::new(),
        String::new(),
    ])
}

/// `[family name, given name, additional names, honorific prefixes, honorific suffixes]`.
pub fn vcard4_name_to_prose_xmpp_name(name: &Vec<String>) -> prose_xmpp_properties::Name {
    let mut name_parts = name.iter();
    prose_xmpp_properties::Name {
        surname: name_parts.next().cloned(),
        given: name_parts.next().cloned(),
        additional: name_parts.next().cloned(),
    }
}
