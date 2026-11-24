//! GayserMC auth/identity hooks for Horizon test servers.
//!
//! `player_login` gates who may join an offline server by `GAYSERMC_ALLOWED_NAMES`
//! (comma separated; empty means accept everyone). `player_login_profile` gives
//! offline clients, which have no account textures, a deterministic skin so a
//! test client renders as an actual player.
//!
//! These are the hooks the mod API exposes at login. They run after the
//! server's own name/whitelist/ban checks; they cannot replace account
//! verification in online mode. See VENDOR.md at the fork root.

use mod_api::{
    self, Flow, Hooks, PlayerLoginEvent, PlayerLoginProfileEvent, ProfileProperty, ServerHost, Text,
};
use std::sync::OnceLock;

/// A 1x1 slim skin, base64 of the `textures` JSON, in vanilla's wire format.
const TEST_SKIN: &str = "eyJ0ZXh0dXJlcyI6eyJTS0lOIjp7InVybCI6Imh0dHA6Ly90ZXh0dXJlcy5taW5lY3JhZnQubmV0L3RleHR1cmUvMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwMDAwIn19fQ";

static ALLOWED: OnceLock<Vec<String>> = OnceLock::new();

fn allowed_names() -> &'static [String] {
    ALLOWED.get_or_init(|| {
        std::env::var("GAYSERMC_ALLOWED_NAMES")
            .unwrap_or_default()
            .split(',')
            .map(str::trim)
            .filter(|name| !name.is_empty())
            .map(str::to_owned)
            .collect()
    })
}

#[derive(Default)]
struct GayserMcAuth;

#[mod_api::horizon_mod]
impl Hooks for GayserMcAuth {
    fn player_login(
        &self,
        _host: &mut dyn ServerHost,
        event: &mut PlayerLoginEvent<'_>,
    ) -> Flow {
        let allowed = allowed_names();
        if !allowed.is_empty()
            && !allowed
                .iter()
                .any(|name| name.eq_ignore_ascii_case(event.name))
        {
            *event.reason = Some(Text::literal(
                "This GayserMC test server has an allowlist".to_owned(),
            ));
            return Flow::Cancel;
        }
        Flow::Continue
    }

    fn player_login_profile(&self, event: &mut PlayerLoginProfileEvent<'_>) {
        if !event
            .properties
            .iter()
            .any(|property| property.name == "textures")
        {
            event.properties.push(ProfileProperty::new(
                "textures".to_owned(),
                TEST_SKIN.to_owned(),
                None,
            ));
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use mod_api::testing::with_mods;

    #[test]
    fn an_offline_profile_gets_a_test_skin() {
        let mut properties: Vec<ProfileProperty> = Vec::new();
        with_mods(vec![mod_api::ModEntry::new("gaysermc-auth", hooks())].into(), || {
            mod_api::dispatch_player_login_profile(7, "Tester", &mut properties);
        });
        let textures: Vec<_> = properties
            .iter()
            .filter(|property| property.name == "textures")
            .collect();
        assert_eq!(textures.len(), 1);
        assert_eq!(textures[0].value, TEST_SKIN);
    }

    #[test]
    fn an_account_skin_is_kept() {
        let mut properties = vec![ProfileProperty::new(
            "textures".to_owned(),
            "account".to_owned(),
            Some("signed".to_owned()),
        )];
        with_mods(vec![mod_api::ModEntry::new("gaysermc-auth", hooks())].into(), || {
            mod_api::dispatch_player_login_profile(7, "Alex", &mut properties);
        });
        assert_eq!(properties.len(), 1);
        assert_eq!(properties[0].value, "account");
    }
}
