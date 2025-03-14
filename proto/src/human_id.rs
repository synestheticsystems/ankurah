#[rustfmt::skip]
pub const WORDLIST: &[&str; 256] = &[
    "ack", "alabama", "alanine", "alaska", "alpha", "angel", "apart", "april",
    "arizona", "arkansas", "artist", "asparagus", "aspen", "august", "autumn",
    "avocado", "bacon", "bakerloo", "batman", "beer", "berlin", "beryllium",
    "black", "blossom", "blue", "bluebird", "bravo", "bulldog", "burger",
    "butter", "california", "carbon", "cardinal", "carolina", "carpet", "cat",
    "ceiling", "charlie", "chicken", "coffee", "cola", "cold", "colorado",
    "comet", "connecticut", "crazy", "cup", "dakota", "december", "delaware",
    "delta", "diet", "don", "double", "early", "earth", "east", "echo",
    "edward", "eight", "eighteen", "eleven", "emma", "enemy", "equal",
    "failed", "fanta", "fifteen", "fillet", "finch", "fish", "five", "fix",
    "floor", "florida", "football", "four", "fourteen", "foxtrot", "freddie",
    "friend", "fruit", "gee", "georgia", "glucose", "golf", "green", "grey",
    "hamper", "happy", "harry", "hawaii", "helium", "high", "hot", "hotel",
    "hydrogen", "idaho", "illinois", "india", "indigo", "ink", "iowa",
    "island", "item", "jersey", "jig", "johnny", "juliet", "july", "jupiter",
    "kansas", "kentucky", "kilo", "king", "kitten", "lactose", "lake", "lamp",
    "lemon", "leopard", "lima", "lion", "lithium", "london", "louisiana",
    "low", "magazine", "magnesium", "maine", "mango", "march", "mars",
    "maryland", "massachusetts", "may", "mexico", "michigan", "mike",
    "minnesota", "mirror", "mississippi", "missouri", "mobile", "mockingbird",
    "monkey", "montana", "moon", "mountain", "muppet", "music", "nebraska",
    "neptune", "network", "nevada", "nine", "nineteen", "nitrogen", "north",
    "november", "nuts", "october", "ohio", "oklahoma", "one", "orange",
    "oranges", "oregon", "oscar", "oven", "oxygen", "papa", "paris", "pasta",
    "pennsylvania", "pip", "pizza", "pluto", "potato", "princess", "purple",
    "quebec", "queen", "quiet", "red", "river", "robert", "robin", "romeo",
    "rugby", "sad", "salami", "saturn", "september", "seven", "seventeen",
    "shade", "sierra", "single", "sink", "six", "sixteen", "skylark", "snake",
    "social", "sodium", "solar", "south", "spaghetti", "speaker", "spring",
    "stairway", "steak", "stream", "summer", "sweet", "table", "tango", "ten",
    "tennessee", "tennis", "texas", "thirteen", "three", "timing", "triple",
    "twelve", "twenty", "two", "uncle", "undress", "uniform", "uranus", "utah",
    "vegan", "venus", "vermont", "victor", "video", "violet", "virginia",
    "washington", "west", "whiskey", "white", "william", "winner", "winter",
    "wisconsin", "wolfram", "wyoming", "xray", "yankee", "yellow", "zebra",
    "zulu" ];

fn compress(bytes: &[u8], target: usize) -> Vec<u8> {
    let seg_size = bytes.len() / target;
    bytes.chunks(seg_size).map(|c| c.iter().fold(0u8, |acc, &x| acc ^ x)).collect::<Vec<u8>>()
}

pub fn humanize(bytes: impl AsRef<[u8]>, words_out: usize) -> String {
    compress(bytes.as_ref(), words_out).iter().map(|&x| WORDLIST[x as usize].to_string()).collect::<Vec<String>>().join("-")
}

pub fn hex(bytes: impl AsRef<[u8]>) -> String { bytes.as_ref().iter().map(|&x| format!("{:x?}", x)).collect::<Vec<String>>().join("") }
