const EPISODES_NAMES: [&'static str; 70] = [
    "datassette",
    "sunjammer",
    "datassette",
    "com_truise",
    "abe_mangger",
    "gods_of_the_new_age",
    "tahlhoff_garten_and_untitled",
    "connectedness_locus",
    "datassette",
    "unity_gain_temple",
    "miles_tilmann",
    "forgotten_light",
    "matt_whitehead",
    "tahlhoff_garten_and_untitled",
    "dan_adeyemi",
    "silent_stelios",
    "graphplan",
    "konx_om_pax",
    "hivemind",
    "uberdog",
    "idol_eyes",
    "mindaugaszq",
    "panda_magic",
    "rites",
    "_nono_",
    "abstraction",
    "michael_hicks",
    "big_war",
    "luke_handsfree",
    "matt_kruse",
    "datassette",
    "chris_seddon",
    "uuav",
    "chukus",
    "nadim_kobeissi",
    "ea7_dmz",
    "lackluster",
    "j_s_aurelius",
    "kidding_kurrys",
    "mark_schneider",
    "sunjammer",
    "datassette",
    "hey_exit",
    "hukka",
    "ehohroma",
    "jo_johnson",
    "abe_mangger",
    "michael_hicks",
    "julien_mier",
    "misc.works",
    "m%C3%BCcha",
    "inchindown",
    "beb_welten",
    "hler",
    "20_jazz_funk_greats",
    "forest_drive_west",
    "hainbach",
    "olive_is_the_sun",
    "miunau",
    "tundra",
    "linnley",
    "our_grey_lives",
    "t-flx",
    "strepsil",
    "matt_whitehead",
    "conrad_clipper",
    "datassette",
    "no_data_available",
    "pearl_river_sound",
    "things_disappear",
];

pub fn get_episode_link(episode: i32) -> String {
    let episode_name = EPISODES_NAMES.get(episode as usize - 1).unwrap();
    let final_link = format!(
        "https://datashat.net/music_for_programming_{}-{}.mp3",
        &episode, episode_name
    );

    final_link
}
