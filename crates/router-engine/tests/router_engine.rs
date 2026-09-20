use router_engine::{
    ApplicationId, Capabilities, CapabilityAvailability, Command, CommandError, Observation,
    OutputId, RouteFailure, RouteStatus, RouterEngine, UnavailableReason,
};

fn app(value: &str) -> ApplicationId {
    ApplicationId::new(value).unwrap()
}

fn output(value: &str) -> OutputId {
    OutputId::new(value).unwrap()
}

fn observation(applications: &[&str], outputs: &[&str], default: Option<&str>) -> Observation {
    Observation {
        capabilities: Capabilities {
            application_routing: CapabilityAvailability::Available,
        },
        applications: applications.iter().map(|value| app(value)).collect(),
        outputs: outputs.iter().map(|value| output(value)).collect(),
        system_default: default.map(output),
    }
}

#[test]
fn setting_a_route_separates_desired_and_effective_output() {
    let mut engine = RouterEngine::new(observation(
        &["browser"],
        &["speakers", "headset"],
        Some("speakers"),
    ));

    let notices = engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();

    let application = &engine.snapshot().applications[0];
    assert!(notices.is_empty());
    assert_eq!(application.desired_output, Some(output("headset")));
    assert_eq!(application.effective_output, Some(output("headset")));
    assert_eq!(application.route_status, RouteStatus::Applied);
}

#[test]
fn missing_desired_output_falls_back_without_forgetting_intent() {
    let mut engine = RouterEngine::new(observation(&["browser"], &["speakers"], Some("speakers")));

    let notices = engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();

    let application = &engine.snapshot().applications[0];
    assert_eq!(application.desired_output, Some(output("headset")));
    assert_eq!(application.effective_output, Some(output("speakers")));
    assert_eq!(
        application.route_status,
        RouteStatus::Fallback(RouteFailure::DesiredOutputUnavailable(output("headset")))
    );
    assert_eq!(notices.len(), 1);
}

#[test]
fn exact_desired_output_is_restored_when_it_returns() {
    let mut engine = RouterEngine::new(observation(&["browser"], &["speakers"], Some("speakers")));
    engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();

    let notices = engine.observe(observation(
        &["browser"],
        &["speakers", "headset"],
        Some("speakers"),
    ));

    let application = &engine.snapshot().applications[0];
    assert!(notices.is_empty());
    assert_eq!(application.desired_output, Some(output("headset")));
    assert_eq!(application.effective_output, Some(output("headset")));
    assert_eq!(application.route_status, RouteStatus::Applied);
}

#[test]
fn following_system_default_clears_the_persistent_route() {
    let mut engine = RouterEngine::new(observation(
        &["browser"],
        &["speakers", "headset"],
        Some("speakers"),
    ));
    engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();

    engine
        .dispatch(Command::FollowSystemDefault {
            application: app("browser"),
        })
        .unwrap();
    engine.observe(observation(&["browser"], &["dock"], Some("dock")));

    let application = &engine.snapshot().applications[0];
    assert_eq!(application.desired_output, None);
    assert_eq!(application.effective_output, Some(output("dock")));
    assert_eq!(
        application.route_status,
        RouteStatus::FollowingSystemDefault
    );
}

#[test]
fn configured_application_is_retained_while_inactive() {
    let mut engine = RouterEngine::new(observation(&["browser"], &["headset"], Some("headset")));
    engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();

    engine.observe(observation(&[], &["headset"], Some("headset")));

    let application = &engine.snapshot().applications[0];
    assert!(!application.active);
    assert_eq!(application.desired_output, Some(output("headset")));
    assert_eq!(application.effective_output, None);
    assert_eq!(application.route_status, RouteStatus::Inactive);
}

#[test]
fn unsupported_routing_is_rejected_before_mutation() {
    let mut initial = observation(&["browser"], &["speakers"], Some("speakers"));
    initial.capabilities.application_routing =
        CapabilityAvailability::Unavailable(UnavailableReason::UnsupportedByPlatform);
    let mut engine = RouterEngine::new(initial);

    let error = engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("speakers"),
        })
        .unwrap_err();

    assert_eq!(
        error,
        CommandError::ApplicationRoutingUnavailable(CapabilityAvailability::Unavailable(
            UnavailableReason::UnsupportedByPlatform
        ))
    );
    assert_eq!(engine.snapshot().applications[0].desired_output, None);
}

#[test]
fn identical_failure_is_not_notified_again_until_it_recovers() {
    let missing = observation(&["browser"], &["speakers"], Some("speakers"));
    let mut engine = RouterEngine::new(missing.clone());

    let first = engine
        .dispatch(Command::SetRoute {
            application: app("browser"),
            output: output("headset"),
        })
        .unwrap();
    let duplicate = engine.observe(missing.clone());
    engine.observe(observation(
        &["browser"],
        &["speakers", "headset"],
        Some("speakers"),
    ));
    let next_episode = engine.observe(missing);

    assert_eq!(first.len(), 1);
    assert!(duplicate.is_empty());
    assert_eq!(next_episode.len(), 1);
}
