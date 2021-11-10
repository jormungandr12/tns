use base64;
use chrono::{DateTime, NaiveDateTime, Utc};
use hex;
use tiny_keccak::Keccak;
use unicode_segmentation::UnicodeSegmentation;

pub fn keccak256(i: &[u8]) -> Vec<u8> {
    let mut o = vec![0u8; 32];
    Keccak::keccak256(i, &mut o);
    return o;
}

pub fn get_label_from_name(name: &String) -> Vec<u8> {
    keccak256(name.as_bytes())
}

pub fn get_token_id_from_label(label: &Vec<u8>) -> String {
    hex::encode(label)
}

pub fn namehash(name: &str) -> Vec<u8> {
    let mut node = vec![0u8; 32];
    if name.is_empty() {
        return node;
    }
    let mut labels: Vec<&str> = name.split(".").collect();
    labels.reverse();
    for label in labels.iter() {
        let mut labelhash = [0u8; 32];
        Keccak::keccak256(label.as_bytes(), &mut labelhash);
        node.append(&mut labelhash.to_vec());
        labelhash = [0u8; 32];
        Keccak::keccak256(node.as_slice(), &mut labelhash);
        node = labelhash.to_vec();
    }
    node
}

pub fn convert_namehash_to_hex_string(namehash: Vec<u8>) -> String {
    hex::encode(namehash)
}

const COLORS: &'static [&'static [&str]] = &[
    &["#F5A4C7", "#F5A4C7", "#FF6483"],
    &["#A1A3A5", "#F5A4C7", "#636466"],
    &["#ABAAF9", "#ABAAF9", "#2A4EF5"],
    &["#EEB9C3", "#F5A4C7", "#EA3323"],
    &["#FBE890", "#F5A4C7", "#FBBE2B"],
    &["#E0CBF2", "#F5A4C7", "#A262F7"],
    &["#A5E2FC", "#F5A4C7", "#6AE0DE"],
    &["#E5B38B", "#F5A4C7", "#AC7240"],
    &["#F8D2A0", "#F9D1A1", "#EE8130"],
    &["#A2CFB3", "#F5A4C7", "#509B7D"],
];

const N_LINE_LETTERS: usize = 14;

pub fn generate_image(name: String, timestamp: u64) -> String {
    let hash = namehash(&name);
    let n_color = COLORS.len() as u8;
    let random_number = hash[0];
    let color = COLORS[(random_number % n_color) as usize];
    let dt = DateTime::<Utc>::from_utc(NaiveDateTime::from_timestamp(timestamp as i64, 0), Utc);
    let date = dt.format("%d.%m.%Y").to_string();
    let graphemes = name.graphemes(true);
    let names: Vec<String> = graphemes
        .collect::<Vec<&str>>()
        .chunks(N_LINE_LETTERS)
        .map(|str_vec: &[&str]| str_vec.join(""))
        .collect::<Vec<String>>();
    let n_line = names.len();
    let mut name_tags = String::from("");
    for i in 0..n_line {
        let name = names[i].clone();
        let y = if n_line == 1 {
            245
        } else {
            278 - 27 * (n_line - 1) + 55 * i
        };
        let name_tag = format!(
            r###"
            <text dominant-baseline="middle" y="{y}" transform-origin="left center" text-rendering="optimizeSpeed" fill="white" font-family="'Courier New', monospace" font-size="48px" font-weight="bolder">
                {name}
            </text>
            "###,
            name = name,
            y = y
        );
        name_tags += &name_tag;
    }

    return String::from("data:image/svg+xml;base64,")
        + &base64::encode(format!(
            r###"
        <svg width="500" height="500" viewBox="0 0 500 500" fill="none" xmlns="http://www.w3.org/2000/svg">
            <path d="M500 0H0V500H500V0Z" fill="{c0}" />
            <mask id="m0" style="mask-type:alpha" maskUnits="userSpaceOnUse" x="0" y="0" width="500" height="500">
                <path d="M500 0H0V500H500V0Z" fill="{c1}" />
            </mask>
            <g mask="url(#m0)">
                <circle cx="658.998" cy="37.9993" r="341.475" stroke="white" stroke-opacity="0.3" stroke-width="1.05057" />
                <circle cx="123" cy="281.995" r="341.475" stroke="white" stroke-width="1.05057" />
                <circle cx="383.998" cy="-233.003" r="341.475" stroke="white" stroke-opacity="0.3" stroke-width="1.05057" />
                <circle cx="101.999" cy="-313.002" r="341.475" stroke="white" stroke-opacity="0.3" stroke-width="1.05057" />
                <path d="M416.138 111.282C418.424 111.282 420.278 109.428 420.278 107.142C420.278 104.855 418.424 103.002 416.138 103.002C413.851 103.002 411.998 104.855 411.998 107.142C411.998 109.428 413.851 111.282 416.138 111.282Z" fill="white" />
            </g>
            <g filter="url(#f0)">
                <circle cx="250.501" cy="249.496" r="127.5" fill="{c2}" />
            </g>
            <path d="M53.8719 75.9698H56.0042V59.2123H62.319V57.1893H47.5571V59.2123H53.8719V75.9698ZM79.1827 59.2123V57.1893H65.7056V75.9698H79.1827V73.9469H67.8652V67.4133H78.0618V65.3904H67.8652V59.2123H79.1827ZM97.3878 63.2035C97.3878 59.677 94.3534 57.1893 90.4715 57.1893H82.8445V75.9698H84.9768V69.2176H89.9521L94.7087 75.9698H97.1417L92.2758 69.0262C95.2555 68.3975 97.3878 66.1558 97.3878 63.2035ZM84.9768 59.2123H90.0888C93.1505 59.2123 95.2281 60.6885 95.2281 63.2035C95.2281 65.7184 93.1505 67.1946 90.0888 67.1946H84.9768V59.2123ZM115.167 63.2035C115.167 59.677 112.133 57.1893 108.251 57.1893H100.624V75.9698H102.756V69.2176H107.732L112.488 75.9698H114.921L110.055 69.0262C113.035 68.3975 115.167 66.1558 115.167 63.2035ZM102.756 59.2123H107.868C110.93 59.2123 113.008 60.6885 113.008 63.2035C113.008 65.7184 110.93 67.1946 107.868 67.1946H102.756V59.2123ZM133.685 75.9698H136.009L127.726 57.1893H125.402L117.119 75.9698H119.443L121.329 71.7052H131.799L133.685 75.9698ZM122.204 69.6823L126.578 59.8137L130.924 69.6823H122.204ZM48.8966 102.97H51.0289V87.6885L62.6744 102.97H64.8067V84.1893H62.6744V99.4707L51.0289 84.1893H48.8966V102.97ZM84.8044 102.97H87.1281L78.845 84.1893H76.5214L68.2383 102.97H70.5619L72.4482 98.7052H82.9182L84.8044 102.97ZM73.3229 96.6823L77.6969 86.8137L82.0434 96.6823H73.3229ZM90.5693 102.97H92.7016V88.7819L99.6725 101.876L106.643 88.7819V102.97H108.776V84.1893H106.643L99.6725 97.2837L92.7016 84.1893H90.5693V102.97ZM126.969 86.2123V84.1893H113.492V102.97H126.969V100.947H115.651V94.4133H125.848V92.3904H115.651V86.2123H126.969ZM54.9927 130.27C59.3666 130.27 61.6082 127.838 61.6082 124.858C61.6082 121.55 59.0386 120.265 55.4574 119.418C52.259 118.652 50.4001 117.969 50.4001 116.028C50.4001 114.306 52.095 112.775 54.4186 112.775C56.3869 112.775 58.2458 113.677 59.804 115.181L61.1162 113.513C59.3939 111.927 57.3163 110.834 54.528 110.834C50.9195 110.834 48.2405 113.185 48.2405 116.192C48.2405 119.445 50.6188 120.621 54.3913 121.495C57.6717 122.261 59.4486 123.054 59.4486 125.049C59.4486 126.744 57.9724 128.33 55.0747 128.33C52.4504 128.33 50.5095 127.209 48.9239 125.623L47.5844 127.291C49.4706 129.15 51.9036 130.27 54.9927 130.27ZM78.7288 113.212V111.189H65.2517V129.97H78.7288V127.947H67.4113V121.413H77.608V119.39H67.4113V113.212H78.7288ZM96.9339 117.203C96.9339 113.677 93.8995 111.189 90.0177 111.189H82.3907V129.97H84.523V123.218H89.4983L94.2549 129.97H96.6879L91.8219 123.026C94.8016 122.397 96.9339 120.156 96.9339 117.203ZM84.523 113.212H89.635C92.6967 113.212 94.7743 114.688 94.7743 117.203C94.7743 119.718 92.6967 121.195 89.635 121.195H84.523V113.212ZM115.452 111.189L108.344 127.345L101.209 111.189H98.8855L107.169 129.97H109.492L117.775 111.189H115.452ZM120.807 129.97H122.939V111.189H120.807V129.97ZM136.151 130.298C138.748 130.298 141.154 129.177 142.904 127.427L141.455 125.951C140.088 127.4 138.202 128.33 136.151 128.33C132.106 128.33 128.689 124.776 128.689 120.566C128.689 116.383 132.106 112.83 136.151 112.83C138.202 112.83 140.088 113.759 141.455 115.208L142.904 113.732C141.154 111.955 138.748 110.861 136.151 110.861C130.957 110.861 126.529 115.29 126.529 120.566C126.529 125.842 130.957 130.298 136.151 130.298ZM159.325 113.212V111.189H145.848V129.97H159.325V127.947H148.007V121.413H158.204V119.39H148.007V113.212H159.325Z" fill="white" />
            <text y="443" x="47" text-rendering="optimizeSpeed" fill="white" font-family="'Courier New', monospace" font-size="28px" font-weight="400">
                {date}
            </text>
            <g transform="translate(47)">
                {name_tags}
            </g>
            <defs>
                <filter id="f0" x="44.2084" y="43.2035" width="412.586" height="412.586" filterUnits="userSpaceOnUse" color-interpolation-filters="sRGB">
                    <feFlood flood-opacity="0" result="BackgroundImageFix" />
                    <feBlend mode="normal" in="SourceGraphic" in2="BackgroundImageFix" result="shape" />
                    <feGaussianBlur stdDeviation="39.3964" result="effect1_foregroundBlur_672:925" />
                </filter>
            </defs>
        </svg>
    "###,
            date = date,
            c0 = color[0],
            c1 = color[1],
            c2 = color[2],
            name_tags = name_tags
        ));
}
