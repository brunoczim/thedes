use crate::math::rand::{
    weight::{TentWeightFn, Weighted},
    Seed,
};
use rand::{distributions::weighted::WeightedIndex, rngs::StdRng, Rng};
use std::mem;

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

    const CONSONANT_WEIGHTS: &'static [Weighted<Self>] = &[
        Weighted { data: Phone::TenuisBilabialStop, weight: 5 },
        Weighted { data: Phone::TenuisAlveolarStop, weight: 3 },
        Weighted { data: Phone::TenuisVelarStop, weight: 5 },
        Weighted { data: Phone::VoicelessAlveolarFricative, weight: 4 },
        Weighted { data: Phone::VoicelessGlottalTransition, weight: 3 },
        Weighted { data: Phone::BilabialNasal, weight: 5 },
        Weighted { data: Phone::AlveolarLateralApproximant, weight: 2 },
        Weighted { data: Phone::PalatalApproximant, weight: 2 },
        Weighted { data: Phone::LabiovelarApproximant, weight: 2 },
    ];

    const VOWEL_WEIGHTS: &'static [Weighted<Self>] = &[
        Weighted { data: Phone::OpenCentralUnroundedVowel, weight: 5 },
        Weighted { data: Phone::MidFrontUnroundedVowel, weight: 3 },
        Weighted { data: Phone::MidBackRoundedVowel, weight: 3 },
        Weighted { data: Phone::ClosedFrontUnroundedVowel, weight: 4 },
        Weighted { data: Phone::ClosedBackRoundedVowel, weight: 4 },
    ];

    fn make_size_weights() -> Vec<Weighted<usize>> {
        TentWeightFn::from_range(
            Self::ALL_PHONES.len() / 2,
            Self::ALL_PHONES.len(),
        )
        .collect()
    }

    fn make_vowel_size_weights() -> Vec<Weighted<usize>> {
        TentWeightFn::from_range(2, Self::VOWEL_WEIGHTS.len() + 1).collect()
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

/// A language's phontactics, i.e. the structure of syllables and words.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Phonotactics {
    /// Count of valid consonants in onset position (before vowel).
    pub onset_count: Vec<Weighted<u8>>,
    /// Count of valid consonants in coda position (after vowel).
    pub coda_count: Vec<Weighted<u8>>,
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

    fn make_onset_start_weights() -> Vec<Weighted<u8>> {
        let iter = TentWeightFn::from_range(
            Self::ONSET_START_MIN,
            Self::ONSET_START_MAX + 1,
        );
        iter.collect()
    }

    fn make_onset_end_weights() -> Vec<Weighted<u8>> {
        let iter = TentWeightFn::from_range(
            Self::ONSET_END_MIN,
            Self::ONSET_END_MAX + 1,
        );
        iter.collect()
    }

    fn make_coda_start_weights() -> Vec<Weighted<u8>> {
        let iter = TentWeightFn::from_range(
            Self::CODA_START_MIN,
            Self::CODA_START_MAX + 1,
        );
        iter.collect()
    }

    fn make_coda_end_weights() -> Vec<Weighted<u8>> {
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
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("onset_start weighted cannot fail");
        let index = rng.sample(&weighted);

        let mut onset_start = weights[index].data;
        let weights = Self::make_onset_end_weights();
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("onset_end weighted cannot fail");
        let index = rng.sample(&weighted);
        let mut onset_end = weights[index].data;

        let weights = Self::make_coda_start_weights();
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("coda_start weighted cannot fail");
        let index = rng.sample(&weighted);
        let mut coda_start = weights[index].data;

        let weights = Self::make_coda_end_weights();
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("coda_end weighted cannot fail");
        let index = rng.sample(&weighted);
        let mut coda_end = weights[index].data;

        if onset_start > onset_end {
            mem::swap(&mut onset_start, &mut onset_end);
        }

        if coda_start > coda_end {
            mem::swap(&mut coda_start, &mut coda_end);
        }

        let mut onset_count =
            Vec::with_capacity((onset_end - onset_start) as usize);
        let weights = TentWeightFn::from_range(onset_start, onset_end)
            .collect::<Vec<_>>();
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("onset weights cannot fail");
        for i in onset_start .. onset_end {
            let index = rng.sample(&weighted);
            onset_count.push(Weighted {
                data: i,
                weight: rng.gen_range(1, (weights[index].weight + 1) * 2),
            });
        }

        let mut coda_count =
            Vec::with_capacity((coda_end - coda_start) as usize);
        let weights =
            TentWeightFn::from_range(coda_start, coda_end).collect::<Vec<_>>();
        let weighted =
            WeightedIndex::new(weights.iter().map(|pair| pair.weight))
                .expect("coda weights cannot fail");
        for i in coda_start .. coda_end {
            let index = rng.sample(&weighted);
            coda_count.push(Weighted {
                data: i,
                weight: rng.gen_range(1, (weights[index].weight + 1) * 2),
            });
        }

        Phonotactics {
            onset_count,
            coda_count,
            double_consonants: rng.gen_bool(Self::DOUBLE_CONSONANTS_PROB),
        }
    }
}

/// A word's phonemes.
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Word {
    pub phonemes: Vec<Phone>,
}

/// A natural language structure (a language of a thede).
#[derive(
    Debug, Clone, PartialEq, Eq, Hash, serde::Serialize, serde::Deserialize,
)]
pub struct Language {
    vowels: Vec<Weighted<Phone>>,
    consonants: Vec<Weighted<Phone>>,
    phonotactics: Phonotactics,
}

impl Language {
    pub fn random(seed: Seed, hash: u64) -> Self {
        let mut rng = seed.make_rng::<_, StdRng>(hash);
        let size_weights = Phone::make_size_weights();
        let weighted =
            WeightedIndex::new(size_weights.iter().map(|pair| pair.weight))
                .expect("total size weights cannot fail");
        let index = rng.sample(weighted);
        let size = size_weights[index].data;

        let size_weights = Phone::make_vowel_size_weights();
        let weighted =
            WeightedIndex::new(size_weights.iter().map(|pair| pair.weight))
                .expect("vowel size weights cannot fail");
        let index = rng.sample(weighted);
        let vowel_size = size_weights[index].data;

        let mut vowels = Vec::with_capacity(size);
        let mut gen_weights = Phone::VOWEL_WEIGHTS.to_vec();

        for _ in 0 .. vowel_size {
            let weighted =
                WeightedIndex::new(gen_weights.iter().map(|pair| pair.weight))
                    .expect("vowel weights cannot fail");
            let index = rng.sample(weighted);
            vowels.push(gen_weights[index]);
            gen_weights.remove(index);
        }

        let mut consonants = Vec::with_capacity(size);
        let mut gen_weights = Phone::CONSONANT_WEIGHTS.to_vec();

        for _ in vowel_size .. size {
            let weighted =
                WeightedIndex::new(gen_weights.iter().map(|pair| pair.weight))
                    .expect("consonant weights cannot fail");
            let index = rng.sample(weighted);
            consonants.push(gen_weights[index]);
            gen_weights.remove(index);
        }

        let phonotactics = Phonotactics::random(&mut rng);

        Self { vowels, consonants, phonotactics }
    }
}
