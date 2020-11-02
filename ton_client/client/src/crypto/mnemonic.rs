/*
* Copyright 2018-2020 TON DEV SOLUTIONS LTD.
*
* Licensed under the SOFTWARE EVALUATION License (the "License"); you may not use
* this file except in compliance with the License.
*
* Unless required by applicable law or agreed to in writing, software
* distributed under the License is distributed on an "AS IS" BASIS,
* WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
* See the License for the specific TON DEV software governing permissions and
* limitations under the License.
*/

use crate::client::{ClientContext};
use crate::crypto;
use crate::crypto::hdkey::HDPrivateKey;
use crate::crypto::internal::{hmac_sha512, key256, pbkdf2_hmac_sha512};
use crate::crypto::keys::KeyPair;
use crate::encoding::hex_decode;
use crate::error::ClientResult;
use bip39::{Language, Mnemonic, MnemonicType};
use ed25519_dalek::{PublicKey, SecretKey};
use hmac::Hmac;
use pbkdf2::pbkdf2;
use rand::RngCore;
use sha2::Sha512;
use crate::crypto::CryptoConfig;

const TON_DICTIONARY: u8 = 0;
const ENGLISH_DICTIONARY: u8 = 1;
const CHINESE_SIMPLIFIED_DICTIONARY: u8 = 2;
const CHINESE_TRADITIONAL_DICTIONARY: u8 = 3;
const FRENCH_DICTIONARY: u8 = 4;
const ITALIAN_DICTIONARY: u8 = 5;
const JAPANESE_DICTIONARY: u8 = 6;
const KOREAN_DICTIONARY: u8 = 7;
const SPANISH_DICTIONARY: u8 = 8;

//---------------------------------------------------------------------------------- mnemonic_words

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfMnemonicWords {
    /// Dictionary identifier
    pub dictionary: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfMnemonicWords {
    /// The list of mnemonic words
    pub words: String,
}

/// Prints the list of words from the specified dictionary
#[api_function]
pub fn mnemonic_words(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfMnemonicWords,
) -> ClientResult<ResultOfMnemonicWords> {
    Ok(ResultOfMnemonicWords {
        words: mnemonics(
            &context.config.crypto,
            params.dictionary,
            Some(context.config.crypto.mnemonic_word_count),
        )?
        .get_words()?,
    })
}

//---------------------------------------------------------------------------- mnemonic_from_random

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfMnemonicFromRandom {
    /// Dictionary identifier
    pub dictionary: Option<u8>,
    /// Mnemonic word count
    pub word_count: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfMnemonicFromRandom {
    /// String of mnemonic words
    pub phrase: String,
}

#[doc(summary = "Generates a random mnemonic")]
/// Generates a random mnemonic from the specified dictionary and word count
#[api_function]
pub fn mnemonic_from_random(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfMnemonicFromRandom,
) -> ClientResult<ResultOfMnemonicFromRandom> {
    Ok(ResultOfMnemonicFromRandom {
        phrase: mnemonics(&context.config.crypto, params.dictionary, params.word_count)?
            .generate_random_phrase()?,
    })
}

//--------------------------------------------------------------------------- mnemonic_from_entropy

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfMnemonicFromEntropy {
    /// Entropy bytes. Hex encoded.
    pub entropy: String,
    /// Dictionary identifier
    pub dictionary: Option<u8>,
    /// Mnemonic word count
    pub word_count: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfMnemonicFromEntropy {
    /// Phrase
    pub phrase: String,
}

#[doc(summary = "Generates mnemonic from the specified entropy")]
/// Generates mnemonic from pre-generated entropy
#[api_function]
pub fn mnemonic_from_entropy(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfMnemonicFromEntropy,
) -> ClientResult<ResultOfMnemonicFromEntropy> {
    let mnemonic = mnemonics(&context.config.crypto, params.dictionary, params.word_count)?;
    Ok(ResultOfMnemonicFromEntropy {
        phrase: mnemonic.phrase_from_entropy(&hex_decode(&params.entropy)?)?,
    })
}

//--------------------------------------------------------------------------------- mnemonic_verify

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfMnemonicVerify {
    /// Phrase
    pub phrase: String,
    /// Dictionary identifier
    pub dictionary: Option<u8>,
    /// Word count
    pub word_count: Option<u8>,
}

#[derive(Serialize, Deserialize, ApiType)]
pub struct ResultOfMnemonicVerify {
    /// Flag indicating the mnemonic is valid or not
    pub valid: bool,
}

#[doc(summary = "Validates a mnemonic phrase")]
/// The phrase supplied will be checked for word length and validated according to the checksum
/// specified in BIP0039.
#[api_function]
pub fn mnemonic_verify(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfMnemonicVerify,
) -> ClientResult<ResultOfMnemonicVerify> {
    let mnemonic = mnemonics(&context.config.crypto, params.dictionary, params.word_count)?;
    Ok(ResultOfMnemonicVerify {
        valid: mnemonic.is_phrase_valid(&params.phrase)?,
    })
}

//----------------------------------------------------------------------- mnemonic_derive_sign_keys

#[derive(Serialize, Deserialize, ApiType)]
pub struct ParamsOfMnemonicDeriveSignKeys {
    /// Phrase
    pub phrase: String,
    /// Derivation path, for instance "m/44'/396'/0'/0/0"
    pub path: Option<String>,
    /// Dictionary identifier
    pub dictionary: Option<u8>,
    /// Word count
    pub word_count: Option<u8>,
}

#[doc(summary = "Derives a key pair for signing from the seed phrase")]
/// Validates the seed phrase, generates master key and then derives
/// the key pair from the master key and the specified path
#[api_function]
pub fn mnemonic_derive_sign_keys(
    context: std::sync::Arc<ClientContext>,
    params: ParamsOfMnemonicDeriveSignKeys,
) -> ClientResult<KeyPair> {
    let mnemonic = mnemonics(&context.config.crypto, params.dictionary, params.word_count)?;
    let path = params
        .path
        .unwrap_or(context.config.crypto.hdkey_derivation_path.clone());
    Ok(mnemonic.derive_ed25519_keys_from_phrase(&context.config.crypto, &params.phrase, &path)?)
}

// Internals

pub(super) fn mnemonics(
    config: &CryptoConfig,
    dictionary: Option<u8>,
    word_count: Option<u8>,
) -> ClientResult<Box<dyn CryptoMnemonic>> {
    let dictionary = dictionary.unwrap_or(config.mnemonic_dictionary);
    let word_count = word_count.unwrap_or(config.mnemonic_word_count);
    if dictionary == TON_DICTIONARY {
        return Ok(Box::new(TonMnemonic::new(word_count)));
    }
    let mnemonic_type = match word_count {
        12 => MnemonicType::Words12,
        15 => MnemonicType::Words15,
        18 => MnemonicType::Words18,
        21 => MnemonicType::Words21,
        24 => MnemonicType::Words24,
        _ => return Err(crypto::Error::bip39_invalid_word_count(word_count)),
    };
    let language = match dictionary {
        ENGLISH_DICTIONARY => Language::English,
        CHINESE_SIMPLIFIED_DICTIONARY => Language::ChineseSimplified,
        CHINESE_TRADITIONAL_DICTIONARY => Language::ChineseTraditional,
        FRENCH_DICTIONARY => Language::French,
        ITALIAN_DICTIONARY => Language::Italian,
        JAPANESE_DICTIONARY => Language::Japanese,
        KOREAN_DICTIONARY => Language::Korean,
        SPANISH_DICTIONARY => Language::Spanish,
        _ => return Err(crypto::Error::bip39_invalid_dictionary(dictionary)),
    };
    Ok(Box::new(Bip39Mnemonic::new(mnemonic_type, language)))
}

pub trait CryptoMnemonic {
    fn get_words(&self) -> ClientResult<String>;
    fn generate_random_phrase(&self) -> ClientResult<String>;
    fn derive_ed25519_keys_from_phrase(
        &self,
        config: &CryptoConfig,
        phrase: &String,
        path: &String,
    ) -> ClientResult<KeyPair>;
    fn phrase_from_entropy(&self, entropy: &[u8]) -> ClientResult<String>;
    fn is_phrase_valid(&self, phrase: &String) -> ClientResult<bool>;
    fn seed_from_phrase_and_salt(&self, phrase: &String, salt: &String) -> ClientResult<String>;
    fn entropy_from_phrase(&self, phrase: &String) -> ClientResult<String>;
}

pub(super) fn check_phrase(mnemonic: &dyn CryptoMnemonic, phrase: &String) -> ClientResult<()> {
    if mnemonic.is_phrase_valid(phrase)? {
        Ok(())
    } else {
        Err(crypto::Error::bip39_invalid_phrase(phrase))
    }
}

pub(crate) struct Bip39Mnemonic {
    mnemonic_type: MnemonicType,
    language: Language,
}

impl Bip39Mnemonic {
    pub(crate) fn new(mnemonic_type: MnemonicType, language: Language) -> Self {
        Bip39Mnemonic {
            mnemonic_type,
            language,
        }
    }
}

fn ed25519_keys_from_secret_bytes(bytes: &[u8]) -> ClientResult<KeyPair> {
    let secret = SecretKey::from_bytes(bytes)
        .map_err(|_| crypto::Error::bip32_invalid_key(&hex::encode(bytes)))?;
    let public = PublicKey::from(&secret);
    Ok(KeyPair::new(
        hex::encode(public.to_bytes()),
        hex::encode(secret.to_bytes()),
    ))
}

impl CryptoMnemonic for Bip39Mnemonic {
    fn get_words(&self) -> ClientResult<String> {
        let words = self.language.wordlist();
        let mut joined = String::new();
        for i in 0..2048 {
            if !joined.is_empty() {
                joined.push(' ');
            }
            joined += words.get_word(i.into());
        }
        Ok(joined)
    }

    fn generate_random_phrase(&self) -> ClientResult<String> {
        let mnemonic = Mnemonic::new(self.mnemonic_type, self.language);
        Ok(mnemonic.phrase().into())
    }

    fn derive_ed25519_keys_from_phrase(
        &self,
        config: &CryptoConfig,
        phrase: &String,
        path: &String,
    ) -> ClientResult<KeyPair> {
        check_phrase(self, phrase)?;
        let derived =
            HDPrivateKey::from_mnemonic(phrase)?.derive_path(path, config.hdkey_compliant)?;
        ed25519_keys_from_secret_bytes(&derived.secret())
    }

    fn phrase_from_entropy(&self, entropy: &[u8]) -> ClientResult<String> {
        let mnemonic = Mnemonic::from_entropy(&entropy, self.language)
            .map_err(|err| crypto::Error::bip39_invalid_entropy(err))?;
        Ok(mnemonic.phrase().into())
    }

    fn is_phrase_valid(&self, phrase: &String) -> ClientResult<bool> {
        Ok(Mnemonic::validate(phrase.as_str(), self.language).is_ok())
    }

    fn seed_from_phrase_and_salt(&self, phrase: &String, salt: &String) -> ClientResult<String> {
        check_phrase(self, phrase)?;
        let mnemonic = Mnemonic::from_phrase(phrase, self.language)
            .map_err(|err| crypto::Error::bip39_invalid_phrase(err))?;

        let salt = format!("mnemonic{}", salt);
        let mut seed = vec![0u8; 64];
        pbkdf2::<Hmac<Sha512>>(
            mnemonic.phrase().as_bytes(),
            salt.as_bytes(),
            2048,
            &mut seed,
        );
        Ok(hex::encode(seed))
    }

    #[allow(dead_code)]
    fn entropy_from_phrase(&self, phrase: &String) -> ClientResult<String> {
        check_phrase(self, phrase)?;
        let mnemonic = Mnemonic::from_phrase(phrase, self.language)
            .map_err(|err| crypto::Error::bip39_invalid_phrase(err))?;
        Ok(hex::encode(mnemonic.entropy()))
    }
}

pub(crate) struct TonMnemonic {
    word_count: u8,
}

impl TonMnemonic {
    pub fn new(word_count: u8) -> Self {
        TonMnemonic { word_count }
    }

    fn words_from_bytes(&self, bytes: &[u8]) -> Vec<&str> {
        let mut words = Vec::new();
        for i in 0usize..self.word_count as usize {
            let mut word_i = 0;
            for j in 0usize..11 {
                let offset = i * 11 + j;
                if (bytes[offset / 8] & (1 << (offset & 7)) as u8) != 0 {
                    word_i |= 1 << j;
                }
            }
            words.push(TON_WORDS[word_i]);
        }
        words
    }

    fn entropy_from_string(string: &String) -> [u8; 64] {
        hmac_sha512(string.as_bytes(), &[])
    }

    fn seed_from_string(string: &String, salt: &str, c: usize) -> [u8; 64] {
        let entropy = Self::entropy_from_string(&string);
        pbkdf2_hmac_sha512(&entropy, salt.as_bytes(), c)
    }

    fn is_basic_seed(string: &String) -> bool {
        let seed = Self::seed_from_string(&string, "TON seed version", 100_000 / 256);
        seed[0] == 0
    }

    fn internal_is_phrase_valid(&self, phrase: &String) -> bool {
        let mut count = 0u8;
        for word in phrase.split(" ") {
            if !TON_WORDS.contains(&word) {
                return false;
            }
            count += 1;
        }
        count == self.word_count && Self::is_basic_seed(phrase)
    }
}

impl CryptoMnemonic for TonMnemonic {
    fn get_words(&self) -> ClientResult<String> {
        return Ok(TON_WORDS.join(" ").to_string());
    }

    fn generate_random_phrase(&self) -> ClientResult<String> {
        let max_iterations: i32 = 256 * 20;
        for _ in 0..max_iterations {
            let mut rng = rand::thread_rng();
            let mut rnd: Vec<u8> = Vec::new();
            rnd.resize(((self.word_count as usize) * 11 + 7) / 8, 0);
            rng.fill_bytes(&mut rnd);
            let words = self.words_from_bytes(&rnd);
            let phrase: String = words.join(" ");
            if !Self::is_basic_seed(&phrase) {
                continue;
            }
            return Ok(phrase);
        }
        return Err(crypto::Error::mnemonic_generation_failed());
    }

    fn derive_ed25519_keys_from_phrase(
        &self,
        config: &CryptoConfig,
        phrase: &String,
        path: &String,
    ) -> ClientResult<KeyPair> {
        check_phrase(self, phrase)?;

        let seed = Self::seed_from_string(&phrase, "TON default seed", 100_000);
        let master = HDPrivateKey::master(&key256(&seed[32..])?, &key256(&seed[..32])?);
        let derived = master.derive_path(path, config.hdkey_compliant)?;
        ed25519_keys_from_secret_bytes(&derived.secret())
    }

    fn phrase_from_entropy(&self, entropy: &[u8]) -> ClientResult<String> {
        if entropy.len() != 24 * 11 / 8 {
            return Err(crypto::Error::mnemonic_from_entropy_failed(
                "Invalid entropy size",
            ));
        }
        let phrase = self.words_from_bytes(entropy).join(" ");
        if Self::is_basic_seed(&phrase) {
            Ok(phrase)
        } else {
            Err(crypto::Error::mnemonic_from_entropy_failed(
                "Invalid entropy",
            ))
        }
    }

    fn is_phrase_valid(&self, phrase: &String) -> ClientResult<bool> {
        Ok(self.internal_is_phrase_valid(phrase))
    }

    fn seed_from_phrase_and_salt(&self, phrase: &String, salt: &String) -> ClientResult<String> {
        check_phrase(self, phrase)?;
        Ok(hex::encode(
            Self::seed_from_string(phrase, salt, 100_000).as_ref(),
        ))
    }

    fn entropy_from_phrase(&self, phrase: &String) -> ClientResult<String> {
        check_phrase(self, phrase)?;
        Ok(hex::encode(Self::entropy_from_string(&phrase).as_ref()))
    }
}

const TON_WORDS: [&str; 2048] = [
    "abandon", "ability", "able", "about", "above", "absent", "absorb", "abstract", "absurd",
    "abuse", "access", "accident", "account", "accuse", "achieve", "acid", "acoustic", "acquire",
    "across", "act", "action", "actor", "actress", "actual", "adapt", "add", "addict", "address",
    "adjust", "admit", "adult", "advance", "advice", "aerobic", "affair", "afford", "afraid",
    "again", "age", "agent", "agree", "ahead", "aim", "air", "airport", "aisle", "alarm", "album",
    "alcohol", "alert", "alien", "all", "alley", "allow", "almost", "alone", "alpha", "already",
    "also", "alter", "always", "amateur", "amazing", "among", "amount", "amused", "analyst",
    "anchor", "ancient", "anger", "angle", "angry", "animal", "ankle", "announce", "annual",
    "another", "answer", "antenna", "antique", "anxiety", "any", "apart", "apology", "appear",
    "apple", "approve", "april", "arch", "arctic", "area", "arena", "argue", "arm", "armed",
    "armor", "army", "around", "arrange", "arrest", "arrive", "arrow", "art", "artefact", "artist",
    "artwork", "ask", "aspect", "assault", "asset", "assist", "assume", "asthma", "athlete",
    "atom", "attack", "attend", "attitude", "attract", "auction", "audit", "august", "aunt",
    "author", "auto", "autumn", "average", "avocado", "avoid", "awake", "aware", "away", "awesome",
    "awful", "awkward", "axis", "baby", "bachelor", "bacon", "badge", "bag", "balance", "balcony",
    "ball", "bamboo", "banana", "banner", "bar", "barely", "bargain", "barrel", "base", "basic",
    "basket", "battle", "beach", "bean", "beauty", "because", "become", "beef", "before", "begin",
    "behave", "behind", "believe", "below", "belt", "bench", "benefit", "best", "betray", "better",
    "between", "beyond", "bicycle", "bid", "bike", "bind", "biology", "bird", "birth", "bitter",
    "black", "blade", "blame", "blanket", "blast", "bleak", "bless", "blind", "blood", "blossom",
    "blouse", "blue", "blur", "blush", "board", "boat", "body", "boil", "bomb", "bone", "bonus",
    "book", "boost", "border", "boring", "borrow", "boss", "bottom", "bounce", "box", "boy",
    "bracket", "brain", "brand", "brass", "brave", "bread", "breeze", "brick", "bridge", "brief",
    "bright", "bring", "brisk", "broccoli", "broken", "bronze", "broom", "brother", "brown",
    "brush", "bubble", "buddy", "budget", "buffalo", "build", "bulb", "bulk", "bullet", "bundle",
    "bunker", "burden", "burger", "burst", "bus", "business", "busy", "butter", "buyer", "buzz",
    "cabbage", "cabin", "cable", "cactus", "cage", "cake", "call", "calm", "camera", "camp", "can",
    "canal", "cancel", "candy", "cannon", "canoe", "canvas", "canyon", "capable", "capital",
    "captain", "car", "carbon", "card", "cargo", "carpet", "carry", "cart", "case", "cash",
    "casino", "castle", "casual", "cat", "catalog", "catch", "category", "cattle", "caught",
    "cause", "caution", "cave", "ceiling", "celery", "cement", "census", "century", "cereal",
    "certain", "chair", "chalk", "champion", "change", "chaos", "chapter", "charge", "chase",
    "chat", "cheap", "check", "cheese", "chef", "cherry", "chest", "chicken", "chief", "child",
    "chimney", "choice", "choose", "chronic", "chuckle", "chunk", "churn", "cigar", "cinnamon",
    "circle", "citizen", "city", "civil", "claim", "clap", "clarify", "claw", "clay", "clean",
    "clerk", "clever", "click", "client", "cliff", "climb", "clinic", "clip", "clock", "clog",
    "close", "cloth", "cloud", "clown", "club", "clump", "cluster", "clutch", "coach", "coast",
    "coconut", "code", "coffee", "coil", "coin", "collect", "color", "column", "combine", "come",
    "comfort", "comic", "common", "company", "concert", "conduct", "confirm", "congress",
    "connect", "consider", "control", "convince", "cook", "cool", "copper", "copy", "coral",
    "core", "corn", "correct", "cost", "cotton", "couch", "country", "couple", "course", "cousin",
    "cover", "coyote", "crack", "cradle", "craft", "cram", "crane", "crash", "crater", "crawl",
    "crazy", "cream", "credit", "creek", "crew", "cricket", "crime", "crisp", "critic", "crop",
    "cross", "crouch", "crowd", "crucial", "cruel", "cruise", "crumble", "crunch", "crush", "cry",
    "crystal", "cube", "culture", "cup", "cupboard", "curious", "current", "curtain", "curve",
    "cushion", "custom", "cute", "cycle", "dad", "damage", "damp", "dance", "danger", "daring",
    "dash", "daughter", "dawn", "day", "deal", "debate", "debris", "decade", "december", "decide",
    "decline", "decorate", "decrease", "deer", "defense", "define", "defy", "degree", "delay",
    "deliver", "demand", "demise", "denial", "dentist", "deny", "depart", "depend", "deposit",
    "depth", "deputy", "derive", "describe", "desert", "design", "desk", "despair", "destroy",
    "detail", "detect", "develop", "device", "devote", "diagram", "dial", "diamond", "diary",
    "dice", "diesel", "diet", "differ", "digital", "dignity", "dilemma", "dinner", "dinosaur",
    "direct", "dirt", "disagree", "discover", "disease", "dish", "dismiss", "disorder", "display",
    "distance", "divert", "divide", "divorce", "dizzy", "doctor", "document", "dog", "doll",
    "dolphin", "domain", "donate", "donkey", "donor", "door", "dose", "double", "dove", "draft",
    "dragon", "drama", "drastic", "draw", "dream", "dress", "drift", "drill", "drink", "drip",
    "drive", "drop", "drum", "dry", "duck", "dumb", "dune", "during", "dust", "dutch", "duty",
    "dwarf", "dynamic", "eager", "eagle", "early", "earn", "earth", "easily", "east", "easy",
    "echo", "ecology", "economy", "edge", "edit", "educate", "effort", "egg", "eight", "either",
    "elbow", "elder", "electric", "elegant", "element", "elephant", "elevator", "elite", "else",
    "embark", "embody", "embrace", "emerge", "emotion", "employ", "empower", "empty", "enable",
    "enact", "end", "endless", "endorse", "enemy", "energy", "enforce", "engage", "engine",
    "enhance", "enjoy", "enlist", "enough", "enrich", "enroll", "ensure", "enter", "entire",
    "entry", "envelope", "episode", "equal", "equip", "era", "erase", "erode", "erosion", "error",
    "erupt", "escape", "essay", "essence", "estate", "eternal", "ethics", "evidence", "evil",
    "evoke", "evolve", "exact", "example", "excess", "exchange", "excite", "exclude", "excuse",
    "execute", "exercise", "exhaust", "exhibit", "exile", "exist", "exit", "exotic", "expand",
    "expect", "expire", "explain", "expose", "express", "extend", "extra", "eye", "eyebrow",
    "fabric", "face", "faculty", "fade", "faint", "faith", "fall", "false", "fame", "family",
    "famous", "fan", "fancy", "fantasy", "farm", "fashion", "fat", "fatal", "father", "fatigue",
    "fault", "favorite", "feature", "february", "federal", "fee", "feed", "feel", "female",
    "fence", "festival", "fetch", "fever", "few", "fiber", "fiction", "field", "figure", "file",
    "film", "filter", "final", "find", "fine", "finger", "finish", "fire", "firm", "first",
    "fiscal", "fish", "fit", "fitness", "fix", "flag", "flame", "flash", "flat", "flavor", "flee",
    "flight", "flip", "float", "flock", "floor", "flower", "fluid", "flush", "fly", "foam",
    "focus", "fog", "foil", "fold", "follow", "food", "foot", "force", "forest", "forget", "fork",
    "fortune", "forum", "forward", "fossil", "foster", "found", "fox", "fragile", "frame",
    "frequent", "fresh", "friend", "fringe", "frog", "front", "frost", "frown", "frozen", "fruit",
    "fuel", "fun", "funny", "furnace", "fury", "future", "gadget", "gain", "galaxy", "gallery",
    "game", "gap", "garage", "garbage", "garden", "garlic", "garment", "gas", "gasp", "gate",
    "gather", "gauge", "gaze", "general", "genius", "genre", "gentle", "genuine", "gesture",
    "ghost", "giant", "gift", "giggle", "ginger", "giraffe", "girl", "give", "glad", "glance",
    "glare", "glass", "glide", "glimpse", "globe", "gloom", "glory", "glove", "glow", "glue",
    "goat", "goddess", "gold", "good", "goose", "gorilla", "gospel", "gossip", "govern", "gown",
    "grab", "grace", "grain", "grant", "grape", "grass", "gravity", "great", "green", "grid",
    "grief", "grit", "grocery", "group", "grow", "grunt", "guard", "guess", "guide", "guilt",
    "guitar", "gun", "gym", "habit", "hair", "half", "hammer", "hamster", "hand", "happy",
    "harbor", "hard", "harsh", "harvest", "hat", "have", "hawk", "hazard", "head", "health",
    "heart", "heavy", "hedgehog", "height", "hello", "helmet", "help", "hen", "hero", "hidden",
    "high", "hill", "hint", "hip", "hire", "history", "hobby", "hockey", "hold", "hole", "holiday",
    "hollow", "home", "honey", "hood", "hope", "horn", "horror", "horse", "hospital", "host",
    "hotel", "hour", "hover", "hub", "huge", "human", "humble", "humor", "hundred", "hungry",
    "hunt", "hurdle", "hurry", "hurt", "husband", "hybrid", "ice", "icon", "idea", "identify",
    "idle", "ignore", "ill", "illegal", "illness", "image", "imitate", "immense", "immune",
    "impact", "impose", "improve", "impulse", "inch", "include", "income", "increase", "index",
    "indicate", "indoor", "industry", "infant", "inflict", "inform", "inhale", "inherit",
    "initial", "inject", "injury", "inmate", "inner", "innocent", "input", "inquiry", "insane",
    "insect", "inside", "inspire", "install", "intact", "interest", "into", "invest", "invite",
    "involve", "iron", "island", "isolate", "issue", "item", "ivory", "jacket", "jaguar", "jar",
    "jazz", "jealous", "jeans", "jelly", "jewel", "job", "join", "joke", "journey", "joy", "judge",
    "juice", "jump", "jungle", "junior", "junk", "just", "kangaroo", "keen", "keep", "ketchup",
    "key", "kick", "kid", "kidney", "kind", "kingdom", "kiss", "kit", "kitchen", "kite", "kitten",
    "kiwi", "knee", "knife", "knock", "know", "lab", "label", "labor", "ladder", "lady", "lake",
    "lamp", "language", "laptop", "large", "later", "latin", "laugh", "laundry", "lava", "law",
    "lawn", "lawsuit", "layer", "lazy", "leader", "leaf", "learn", "leave", "lecture", "left",
    "leg", "legal", "legend", "leisure", "lemon", "lend", "length", "lens", "leopard", "lesson",
    "letter", "level", "liar", "liberty", "library", "license", "life", "lift", "light", "like",
    "limb", "limit", "link", "lion", "liquid", "list", "little", "live", "lizard", "load", "loan",
    "lobster", "local", "lock", "logic", "lonely", "long", "loop", "lottery", "loud", "lounge",
    "love", "loyal", "lucky", "luggage", "lumber", "lunar", "lunch", "luxury", "lyrics", "machine",
    "mad", "magic", "magnet", "maid", "mail", "main", "major", "make", "mammal", "man", "manage",
    "mandate", "mango", "mansion", "manual", "maple", "marble", "march", "margin", "marine",
    "market", "marriage", "mask", "mass", "master", "match", "material", "math", "matrix",
    "matter", "maximum", "maze", "meadow", "mean", "measure", "meat", "mechanic", "medal", "media",
    "melody", "melt", "member", "memory", "mention", "menu", "mercy", "merge", "merit", "merry",
    "mesh", "message", "metal", "method", "middle", "midnight", "milk", "million", "mimic", "mind",
    "minimum", "minor", "minute", "miracle", "mirror", "misery", "miss", "mistake", "mix", "mixed",
    "mixture", "mobile", "model", "modify", "mom", "moment", "monitor", "monkey", "monster",
    "month", "moon", "moral", "more", "morning", "mosquito", "mother", "motion", "motor",
    "mountain", "mouse", "move", "movie", "much", "muffin", "mule", "multiply", "muscle", "museum",
    "mushroom", "music", "must", "mutual", "myself", "mystery", "myth", "naive", "name", "napkin",
    "narrow", "nasty", "nation", "nature", "near", "neck", "need", "negative", "neglect",
    "neither", "nephew", "nerve", "nest", "net", "network", "neutral", "never", "news", "next",
    "nice", "night", "noble", "noise", "nominee", "noodle", "normal", "north", "nose", "notable",
    "note", "nothing", "notice", "novel", "now", "nuclear", "number", "nurse", "nut", "oak",
    "obey", "object", "oblige", "obscure", "observe", "obtain", "obvious", "occur", "ocean",
    "october", "odor", "off", "offer", "office", "often", "oil", "okay", "old", "olive", "olympic",
    "omit", "once", "one", "onion", "online", "only", "open", "opera", "opinion", "oppose",
    "option", "orange", "orbit", "orchard", "order", "ordinary", "organ", "orient", "original",
    "orphan", "ostrich", "other", "outdoor", "outer", "output", "outside", "oval", "oven", "over",
    "own", "owner", "oxygen", "oyster", "ozone", "pact", "paddle", "page", "pair", "palace",
    "palm", "panda", "panel", "panic", "panther", "paper", "parade", "parent", "park", "parrot",
    "party", "pass", "patch", "path", "patient", "patrol", "pattern", "pause", "pave", "payment",
    "peace", "peanut", "pear", "peasant", "pelican", "pen", "penalty", "pencil", "people",
    "pepper", "perfect", "permit", "person", "pet", "phone", "photo", "phrase", "physical",
    "piano", "picnic", "picture", "piece", "pig", "pigeon", "pill", "pilot", "pink", "pioneer",
    "pipe", "pistol", "pitch", "pizza", "place", "planet", "plastic", "plate", "play", "please",
    "pledge", "pluck", "plug", "plunge", "poem", "poet", "point", "polar", "pole", "police",
    "pond", "pony", "pool", "popular", "portion", "position", "possible", "post", "potato",
    "pottery", "poverty", "powder", "power", "practice", "praise", "predict", "prefer", "prepare",
    "present", "pretty", "prevent", "price", "pride", "primary", "print", "priority", "prison",
    "private", "prize", "problem", "process", "produce", "profit", "program", "project", "promote",
    "proof", "property", "prosper", "protect", "proud", "provide", "public", "pudding", "pull",
    "pulp", "pulse", "pumpkin", "punch", "pupil", "puppy", "purchase", "purity", "purpose",
    "purse", "push", "put", "puzzle", "pyramid", "quality", "quantum", "quarter", "question",
    "quick", "quit", "quiz", "quote", "rabbit", "raccoon", "race", "rack", "radar", "radio",
    "rail", "rain", "raise", "rally", "ramp", "ranch", "random", "range", "rapid", "rare", "rate",
    "rather", "raven", "raw", "razor", "ready", "real", "reason", "rebel", "rebuild", "recall",
    "receive", "recipe", "record", "recycle", "reduce", "reflect", "reform", "refuse", "region",
    "regret", "regular", "reject", "relax", "release", "relief", "rely", "remain", "remember",
    "remind", "remove", "render", "renew", "rent", "reopen", "repair", "repeat", "replace",
    "report", "require", "rescue", "resemble", "resist", "resource", "response", "result",
    "retire", "retreat", "return", "reunion", "reveal", "review", "reward", "rhythm", "rib",
    "ribbon", "rice", "rich", "ride", "ridge", "rifle", "right", "rigid", "ring", "riot", "ripple",
    "risk", "ritual", "rival", "river", "road", "roast", "robot", "robust", "rocket", "romance",
    "roof", "rookie", "room", "rose", "rotate", "rough", "round", "route", "royal", "rubber",
    "rude", "rug", "rule", "run", "runway", "rural", "sad", "saddle", "sadness", "safe", "sail",
    "salad", "salmon", "salon", "salt", "salute", "same", "sample", "sand", "satisfy", "satoshi",
    "sauce", "sausage", "save", "say", "scale", "scan", "scare", "scatter", "scene", "scheme",
    "school", "science", "scissors", "scorpion", "scout", "scrap", "screen", "script", "scrub",
    "sea", "search", "season", "seat", "second", "secret", "section", "security", "seed", "seek",
    "segment", "select", "sell", "seminar", "senior", "sense", "sentence", "series", "service",
    "session", "settle", "setup", "seven", "shadow", "shaft", "shallow", "share", "shed", "shell",
    "sheriff", "shield", "shift", "shine", "ship", "shiver", "shock", "shoe", "shoot", "shop",
    "short", "shoulder", "shove", "shrimp", "shrug", "shuffle", "shy", "sibling", "sick", "side",
    "siege", "sight", "sign", "silent", "silk", "silly", "silver", "similar", "simple", "since",
    "sing", "siren", "sister", "situate", "six", "size", "skate", "sketch", "ski", "skill", "skin",
    "skirt", "skull", "slab", "slam", "sleep", "slender", "slice", "slide", "slight", "slim",
    "slogan", "slot", "slow", "slush", "small", "smart", "smile", "smoke", "smooth", "snack",
    "snake", "snap", "sniff", "snow", "soap", "soccer", "social", "sock", "soda", "soft", "solar",
    "soldier", "solid", "solution", "solve", "someone", "song", "soon", "sorry", "sort", "soul",
    "sound", "soup", "source", "south", "space", "spare", "spatial", "spawn", "speak", "special",
    "speed", "spell", "spend", "sphere", "spice", "spider", "spike", "spin", "spirit", "split",
    "spoil", "sponsor", "spoon", "sport", "spot", "spray", "spread", "spring", "spy", "square",
    "squeeze", "squirrel", "stable", "stadium", "staff", "stage", "stairs", "stamp", "stand",
    "start", "state", "stay", "steak", "steel", "stem", "step", "stereo", "stick", "still",
    "sting", "stock", "stomach", "stone", "stool", "story", "stove", "strategy", "street",
    "strike", "strong", "struggle", "student", "stuff", "stumble", "style", "subject", "submit",
    "subway", "success", "such", "sudden", "suffer", "sugar", "suggest", "suit", "summer", "sun",
    "sunny", "sunset", "super", "supply", "supreme", "sure", "surface", "surge", "surprise",
    "surround", "survey", "suspect", "sustain", "swallow", "swamp", "swap", "swarm", "swear",
    "sweet", "swift", "swim", "swing", "switch", "sword", "symbol", "symptom", "syrup", "system",
    "table", "tackle", "tag", "tail", "talent", "talk", "tank", "tape", "target", "task", "taste",
    "tattoo", "taxi", "teach", "team", "tell", "ten", "tenant", "tennis", "tent", "term", "test",
    "text", "thank", "that", "theme", "then", "theory", "there", "they", "thing", "this",
    "thought", "three", "thrive", "throw", "thumb", "thunder", "ticket", "tide", "tiger", "tilt",
    "timber", "time", "tiny", "tip", "tired", "tissue", "title", "toast", "tobacco", "today",
    "toddler", "toe", "together", "toilet", "token", "tomato", "tomorrow", "tone", "tongue",
    "tonight", "tool", "tooth", "top", "topic", "topple", "torch", "tornado", "tortoise", "toss",
    "total", "tourist", "toward", "tower", "town", "toy", "track", "trade", "traffic", "tragic",
    "train", "transfer", "trap", "trash", "travel", "tray", "treat", "tree", "trend", "trial",
    "tribe", "trick", "trigger", "trim", "trip", "trophy", "trouble", "truck", "true", "truly",
    "trumpet", "trust", "truth", "try", "tube", "tuition", "tumble", "tuna", "tunnel", "turkey",
    "turn", "turtle", "twelve", "twenty", "twice", "twin", "twist", "two", "type", "typical",
    "ugly", "umbrella", "unable", "unaware", "uncle", "uncover", "under", "undo", "unfair",
    "unfold", "unhappy", "uniform", "unique", "unit", "universe", "unknown", "unlock", "until",
    "unusual", "unveil", "update", "upgrade", "uphold", "upon", "upper", "upset", "urban", "urge",
    "usage", "use", "used", "useful", "useless", "usual", "utility", "vacant", "vacuum", "vague",
    "valid", "valley", "valve", "van", "vanish", "vapor", "various", "vast", "vault", "vehicle",
    "velvet", "vendor", "venture", "venue", "verb", "verify", "version", "very", "vessel",
    "veteran", "viable", "vibrant", "vicious", "victory", "video", "view", "village", "vintage",
    "violin", "virtual", "virus", "visa", "visit", "visual", "vital", "vivid", "vocal", "voice",
    "void", "volcano", "volume", "vote", "voyage", "wage", "wagon", "wait", "walk", "wall",
    "walnut", "want", "warfare", "warm", "warrior", "wash", "wasp", "waste", "water", "wave",
    "way", "wealth", "weapon", "wear", "weasel", "weather", "web", "wedding", "weekend", "weird",
    "welcome", "west", "wet", "whale", "what", "wheat", "wheel", "when", "where", "whip",
    "whisper", "wide", "width", "wife", "wild", "will", "win", "window", "wine", "wing", "wink",
    "winner", "winter", "wire", "wisdom", "wise", "wish", "witness", "wolf", "woman", "wonder",
    "wood", "wool", "word", "work", "world", "worry", "worth", "wrap", "wreck", "wrestle", "wrist",
    "write", "wrong", "yard", "year", "yellow", "you", "young", "youth", "zebra", "zero", "zone",
    "zoo",
];
