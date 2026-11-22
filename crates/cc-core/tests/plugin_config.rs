use cc_core::{
    AlertSinkSelection, AuxSensorSelection, CsiSourceSelection, FallClassifierSelection,
    PluginConfig, PluginConfigError,
};

const FAKE_CONFIG: &str = r#"
csi_source = "fake"
fall_classifier = "fake"
aux_sensor = "fake"
alert_sink = "fake"
"#;

#[test]
fn parses_all_four_selections() {
    assert_eq!(
        PluginConfig::from_toml(FAKE_CONFIG).unwrap(),
        PluginConfig {
            csi_source: CsiSourceSelection::Fake,
            fall_classifier: FallClassifierSelection::Fake,
            aux_sensor: AuxSensorSelection::Fake,
            alert_sink: AlertSinkSelection::Fake,
        }
    );
}

#[test]
fn identifies_the_field_and_name_for_an_unknown_implementation() {
    for field in ["csi_source", "fall_classifier", "aux_sensor", "alert_sink"] {
        let contents = FAKE_CONFIG.replace(
            &format!("{field} = \"fake\""),
            &format!("{field} = \"unknown\""),
        );
        let error = PluginConfig::from_toml(&contents).unwrap_err();

        assert!(matches!(
            &error,
            PluginConfigError::UnknownImplementation { field: actual, name }
                if *actual == field && name == "unknown"
        ));
        assert!(error.to_string().contains(field));
        assert!(error.to_string().contains("unknown"));
    }
}

#[test]
fn rejects_malformed_incomplete_and_unexpected_fields() {
    let with_extra_field = format!("{FAKE_CONFIG}extra = \"fake\"\n");
    for contents in ["csi_source =", "csi_source = 'fake'", &with_extra_field] {
        assert!(matches!(
            PluginConfig::from_toml(contents),
            Err(PluginConfigError::Parse(_))
        ));
    }
}
