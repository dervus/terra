pub fn capitalize<T: AsRef<str>>(input: T) -> String {
    let mut output = String::with_capacity(input.as_ref().len());
    for (index, character) in input.as_ref().to_owned().chars().enumerate() {
        if index == 0 {
            for upcase in character.to_uppercase() {
                output.push(upcase);
            }
        } else {
            output.push(character);
        }
    }
    output
}

pub fn hexstring<T: AsRef<[u8]>>(input: T) -> String {
    use std::fmt::Write;
    let input = input.as_ref();
    let mut output = String::with_capacity(input.len() * 2);
    for byte in input.iter() {
        write!(&mut output, "{:X}", byte).unwrap();
    }
    output
}

pub fn random_footnote() -> &'static str {
    use rand::seq::SliceRandom;
    let strings = [
        "BETTER THAN SKYLAND",
        "A MACHINE FOR SHEEPS",
        "STARVE TOGETHER",
        "THE ROLEPLAY SIMULATOR 2020",
        "TOTAL ROLEPLAY",
        "THIS ROLEPLAY OF MINE",
        "HELL, IT'S ABOUT TIME",
        "LEGACY OF THE BENCH",
        "66 FREE DLC INCLUDED",
        "JUST STOP HITTING UPDATE",
        "DARKEST TAVERN",
        "ROLEPLAY IS STRANGE",
        "HEART OF ROLEPLAY",
        "MASTERS WILL BE WATCHING",
        "AUGUST IS HERE",
        "QUENTAS, PLEASE",
        "GO AND TELL OTHERS WHAT YOU'VE SEEN",
        "WE DRINK YOUR MILKSHAKE",
        "WARCRAFT: WIZARDS AND NOBLES",
    ];
    strings.choose(&mut rand::thread_rng()).unwrap()
}
