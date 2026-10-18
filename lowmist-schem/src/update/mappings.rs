pub(super) fn mapping_strings() -> Vec<(i32, &'static str)> {
    vec![
        // (TARGET_DATA_VERSION, include_str!("mapping-FROMtoTARGET.json"))
        (1952, include_str!("mapping-1.13to1.13.2.json")), // This ViaVersion mapping actually targets 1.14 for some reason
        (2225, include_str!("mapping-1.14to1.15.json")),
        (2566, include_str!("mapping-1.15to1.16.json")),
        (2578, include_str!("mapping-1.16to1.16.2.json")),
        (2724, include_str!("mapping-1.16.2to1.17.json")),
        (3105, include_str!("mapping-1.18to1.19.json")),
        (3463, include_str!("mapping-1.19.4to1.20.json")),
        (3578, include_str!("mapping-1.20to1.20.2.json")),
        (3698, include_str!("mapping-1.20.2to1.20.3.json")),
        (3837, include_str!("mapping-1.20.3to1.20.5.json")),
        (4189, include_str!("mapping-1.21.2to1.21.4.json")),
        (4325, include_str!("mapping-1.21.4to1.21.5.json")),
        (4554, include_str!("mapping-1.21.7to1.21.9.json")),
    ]
}
