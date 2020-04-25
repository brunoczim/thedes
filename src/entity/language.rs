use crate::math::rand::{
    weighted::{self, TentWeightFn},
    Seed,
};
use rand::{rngs::StdRng, Rng};
use std::{
    collections::HashMap,
    fmt::{self, Write},
    mem,
};

const WORD_SEED_SALT: u64 = 0x4D5E19580AB83E25;

type Weight = u64;

/// Possible phones (phonetic sounds) for a thede's language.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Phone {
    /// [p˭]
    TenuisBilabialStop,
    /// [t˭]
    TenuisAlveolarStop,
    /// [k˭]
    TenuisVelarStop,
    /// [s]
    VoicelessAlveolarFricative,
    /// [h]
    VoicelessGlottalTransition,
    /// [m]
    BilabialNasal,
    /// [l]
    AlveolarLateralApproximant,
    /// [j]
    PalatalApproximant,
    /// [w]
    LabiovelarApproximant,
    /// [ä]
    OpenCentralUnroundedVowel,
    /// [e̞]
    MidFrontUnroundedVowel,
    /// [o̞]
    MidBackRoundedVowel,
    /// [i]
    ClosedFrontUnroundedVowel,
    /// [u]
    ClosedBackRoundedVowel,
}

impl Phone {
    const ALL_PHONES: &'static [Self] = &[
        Phone::TenuisBilabialStop,
        Phone::TenuisAlveolarStop,
        Phone::TenuisVelarStop,
        Phone::VoicelessAlveolarFricative,
        Phone::VoicelessGlottalTransition,
        Phone::BilabialNasal,
        Phone::AlveolarLateralApproximant,
        Phone::PalatalApproximant,
        Phone::LabiovelarApproximant,
        Phone::OpenCentralUnroundedVowel,
        Phone::MidFrontUnroundedVowel,
        Phone::MidBackRoundedVowel,
        Phone::ClosedFrontUnroundedVowel,
        Phone::ClosedBackRoundedVowel,
    ];

    const CONSONANT_WEIGHTS: &'static [weighted::Entry<Self, Weight>] = &[
        weighted::Entry { data: Phone::TenuisBilabialStop, weight: 5 },
        weighted::Entry { data: Phone::TenuisAlveolarStop, weight: 3 },
        weighted::Entry { data: Phone::TenuisVelarStop, weight: 5 },
        weighted::Entry { data: Phone::VoicelessAlveolarFricative, weight: 4 },
        weighted::Entry { data: Phone::VoicelessGlottalTransition, weight: 3 },
        weighted::Entry { data: Phone::BilabialNasal, weight: 5 },
        weighted::Entry { data: Phone::AlveolarLateralApproximant, weight: 2 },
        weighted::Entry { data: Phone::PalatalApproximant, weight: 2 },
        weighted::Entry { data: Phone::LabiovelarApproximant, weight: 2 },
    ];

    const VOWEL_WEIGHTS: &'static [weighted::Entry<Self, Weight>] = &[
        weighted::Entry { data: Phone::OpenCentralUnroundedVowel, weight: 5 },
        weighted::Entry { data: Phone::MidFrontUnroundedVowel, weight: 3 },
        weighted::Entry { data: Phone::MidBackRoundedVowel, weight: 3 },
        weighted::Entry { data: Phone::ClosedFrontUnroundedVowel, weight: 4 },
        weighted::Entry { data: Phone::ClosedBackRoundedVowel, weight: 4 },
    ];

    fn make_size_weights() -> Vec<weighted::Entry<usize, Weight>> {
        TentWeightFn::from_range(
            Self::ALL_PHONES.len() / 2,
            Self::ALL_PHONES.len(),
        )
        .collect()
    }

    fn make_vowel_size_weights() -> Vec<weighted::Entry<usize, Weight>> {
        TentWeightFn::from_range(2, Self::VOWEL_WEIGHTS.len() + 1).collect()
    }
}

impl fmt::Display for Phone {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        fmt.pad(match self {
            Phone::TenuisBilabialStop => "p",
            Phone::TenuisAlveolarStop => "t",
            Phone::TenuisVelarStop => "k",
            Phone::VoicelessAlveolarFricative => "s",
            Phone::VoicelessGlottalTransition => "h",
            Phone::BilabialNasal => "m",
            Phone::AlveolarLateralApproximant => "l",
            Phone::PalatalApproximant => "j",
            Phone::LabiovelarApproximant => "w",
            Phone::OpenCentralUnroundedVowel => "ä",
            Phone::MidFrontUnroundedVowel => "e̞",
            Phone::MidBackRoundedVowel => "o̞",
            Phone::ClosedFrontUnroundedVowel => "i",
            Phone::ClosedBackRoundedVowel => "u",
        })
    }
}

/// A possible meaning for a word.
#[derive(
    Debug,
    Clone,
    Copy,
    PartialEq,
    Eq,
    PartialOrd,
    Ord,
    Hash,
    serde::Serialize,
    serde::Deserialize,
)]
pub enum Meaning {
    /// First person singular.
    I,
    /// Verb to be in the meaning of existing.
    Exist,
}

impl Meaning {
    ///All the meanings.
    pub const ALL: &'static [Self] = &[Meaning::I, Meaning::Exist];
}

/// A language's phontactics, i.e. the structure of syllables and words.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Phonotactics {
    /// Count of valid consonants in onset position (before vowel).
    pub onset_count: weighted::Entries<u8, Weight>,
    /// Count of valid consonants in coda position (after vowel).
    pub coda_count: weighted::Entries<u8, Weight>,
    /// Whether double consonants are allowed.
    pub double_consonants: bool,
}

impl Phonotactics {
    const ONSET_START_MIN: u8 = 0;
    const ONSET_START_MAX: u8 = 2;
    const ONSET_END_MIN: u8 = 1;
    const ONSET_END_MAX: u8 = 3;

    const CODA_START_MIN: u8 = 0;
    const CODA_START_MAX: u8 = 2;
    const CODA_END_MIN: u8 = 0;
    const CODA_END_MAX: u8 = 3;

    const DOUBLE_CONSONANTS_PROB: f64 = 0.375;

    fn make_onset_start_weights() -> Vec<weighted::Entry<u8, Weight>> {
        let iter = TentWeightFn::from_range(
            Self::ONSET_START_MIN,
            Self::ONSET_START_MAX + 1,
        );
        iter.collect()
    }

    fn make_onset_end_weights() -> Vec<weighted::Entry<u8, Weight>> {
        let iter = TentWeightFn::from_range(
            Self::ONSET_END_MIN,
            Self::ONSET_END_MAX + 1,
        );
        iter.collect()
    }

    fn make_coda_start_weights() -> Vec<weighted::Entry<u8, Weight>> {
        let iter = TentWeightFn::from_range(
            Self::CODA_START_MIN,
            Self::CODA_START_MAX + 1,
        );
        iter.collect()
    }

    fn make_coda_end_weights() -> Vec<weighted::Entry<u8, Weight>> {
        let iter = TentWeightFn::from_range(
            Self::CODA_END_MIN,
            Self::CODA_END_MAX + 1,
        );
        iter.collect()
    }

    fn random<R>(rng: &mut R) -> Self
    where
        R: Rng,
    {
        let weights = Self::make_onset_start_weights();
        let weighted = weighted::Entries::new(weights.iter().cloned());
        let mut onset_start = rng.sample(&weighted).data;

        let weights = Self::make_onset_end_weights();
        let weighted = weighted::Entries::new(weights.iter().cloned());
        let mut onset_end = rng.sample(&weighted).data;

        let weights = Self::make_coda_start_weights();
        let weighted = weighted::Entries::new(weights.iter().cloned());
        let mut coda_start = rng.sample(&weighted).data;

        let weights = Self::make_coda_end_weights();
        let weighted = weighted::Entries::new(weights.iter().cloned());
        let mut coda_end = rng.sample(&weighted).data;

        if onset_start > onset_end {
            mem::swap(&mut onset_start, &mut onset_end);
        }

        if coda_start > coda_end {
            mem::swap(&mut coda_start, &mut coda_end);
        }

        let mut onset_count =
            Vec::with_capacity((onset_end - onset_start) as usize);
        let weights = TentWeightFn::from_range(onset_start, onset_end)
            .collect::<Vec<weighted::Entry<u8, Weight>>>();
        let weighted = weighted::Entries::new(weights.iter().cloned());
        for i in onset_start .. onset_end {
            let weight = rng.sample(&weighted).weight;
            onset_count.push(weighted::Entry {
                data: i,
                weight: rng.gen_range(1, (weight + 1) * 2),
            });
        }

        let mut coda_count =
            Vec::with_capacity((coda_end - coda_start) as usize);
        let weights = TentWeightFn::from_range(coda_start, coda_end)
            .collect::<Vec<weighted::Entry<u8, Weight>>>();
        let weighted = weighted::Entries::new(weights.iter().map(Clone::clone));
        for i in coda_start .. coda_end {
            let weight = rng.sample(&weighted).weight;
            coda_count.push(weighted::Entry {
                data: i,
                weight: rng.gen_range(1, (weight + 1) * 2),
            });
        }

        Phonotactics {
            onset_count: weighted::Entries::new(onset_count),
            coda_count: weighted::Entries::new(coda_count),
            double_consonants: rng.gen_bool(Self::DOUBLE_CONSONANTS_PROB),
        }
    }
}

/// A word's data.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Word {
    /// Phonemes composing this word.
    pub phones: Vec<Phone>,
}

impl Word {
    const MIN_SYLLABLES: usize = 1;
    const MAX_SYLLABLES: usize = 5;
}

impl fmt::Display for Word {
    fn fmt(&self, fmt: &mut fmt::Formatter) -> fmt::Result {
        let mut buf = String::with_capacity(self.phones.len());

        for &phone in &self.phones {
            write!(buf, "{}", phone)?;
        }

        fmt.pad(&buf)
    }
}

/// A natural language structure (a language of a thede).
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct Language {
    vowels: weighted::Entries<Phone, Weight>,
    consonants: weighted::Entries<Phone, Weight>,
    phonotactics: Phonotactics,
    words: HashMap<Meaning, Word>,
}

impl Language {
    /// Creates a random language.
    pub fn random(seed: Seed, hash: u64) -> Self {
        let mut rng = seed.make_rng::<_, StdRng>(hash);
        let size_weights = Phone::make_size_weights();
        let weighted = weighted::Entries::new(size_weights.iter().cloned());
        let size = rng.sample(&weighted).data;

        let size_weights = Phone::make_vowel_size_weights();
        let weighted = weighted::Entries::new(size_weights.iter().cloned());
        let vowel_size = rng.sample(&weighted).data;

        let mut vowels = Vec::with_capacity(size);
        let mut gen_weights = Phone::VOWEL_WEIGHTS.to_vec();

        for _ in 0 .. vowel_size {
            let weighted = weighted::Indices::new(
                gen_weights.iter().map(|pair| pair.weight),
            );
            let index = rng.sample(weighted);
            vowels.push(gen_weights[index]);
            gen_weights.remove(index);
        }

        let mut consonants = Vec::with_capacity(size);
        let mut gen_weights = Phone::CONSONANT_WEIGHTS.to_vec();

        for _ in vowel_size .. size {
            let weighted = weighted::Indices::new(
                gen_weights.iter().map(|pair| pair.weight),
            );
            let index = rng.sample(&weighted);
            consonants.push(gen_weights[index]);
            gen_weights.remove(index);
        }

        let phonotactics = Phonotactics::random(&mut rng);

        Self {
            vowels: weighted::Entries::new(vowels),
            consonants: weighted::Entries::new(consonants),
            phonotactics,
            words: HashMap::new(),
        }
    }

    /// Generates a word for the given meaning.
    pub fn gen_word(&mut self, meaning: Meaning, seed: Seed) {
        let mut rng = seed.make_rng::<_, StdRng>((WORD_SEED_SALT, meaning));
        let syllables = rng.gen_range(Word::MIN_SYLLABLES, Word::MAX_SYLLABLES);

        let mut word = Word { phones: Vec::with_capacity(syllables * 2) };

        for _ in 0 .. syllables {
            let onset_count = rng.sample(&self.phonotactics.onset_count).data;
            for _ in 0 .. onset_count {
                word.phones.push(rng.sample(&self.consonants).data);
            }

            word.phones.push(rng.sample(&self.vowels).data);

            let coda_count = rng.sample(&self.phonotactics.coda_count).data;
            for _ in 0 .. coda_count {
                word.phones.push(rng.sample(&self.consonants).data);
            }
        }

        self.words.insert(meaning, word);
    }

    /// Gets the word for the given meaning.
    pub fn word_for(&self, meaning: Meaning) -> Option<&Word> {
        self.words.get(&meaning)
    }
}
