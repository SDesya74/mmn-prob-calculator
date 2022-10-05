#![feature(slice_group_by)]

use std::{
    cmp::{max, min},
    error::Error,
    fs::{self, File},
    path::PathBuf,
};

use rand::{rngs::ThreadRng, Rng};

fn roll_dices(amount: u32) -> Vec<u32> {
    let mut rng = rand::thread_rng();
    (0..amount).map(|_| rng.gen_range(1..=6)).collect()
}

fn get_dices_amount_by_skill(skill: u32) -> u32 {
    min(skill, 10)
}

fn add_bonuses_by_skill(rolled_dices: Vec<u32>, skill: u32) -> Vec<u32> {
    let bonus_dices_amount = (max(0, skill as i32 - 10) % 10) as usize;
    let high_bonus = skill / 10;
    let low_bonus = max(high_bonus, 1) - 1;

    // println!("dices with bonuses amount: {}", bonus_dices_amount);
    // println!("addition for bonused dices: {}", high_bonus);
    // println!("addition for remain dices: {}", low_bonus);

    let (high, low) = rolled_dices.as_slice().split_at(bonus_dices_amount);

    high.into_iter()
        .map(|e| e + high_bonus)
        .chain(low.into_iter().map(|e| e + low_bonus))
        .collect()
}

fn roll_dices_by_skill(skill: u32) -> Vec<u32> {
    let total_dices_amount = get_dices_amount_by_skill(skill);
    let rolled_dices = roll_dices(total_dices_amount);
    add_bonuses_by_skill(rolled_dices, skill)
}

fn calc_dices_value(dices: &Vec<u32>) -> f32 {
    let (jun, sen): (Vec<u32>, Vec<u32>) = dices.iter().partition(|&&e| e < 6);

    if sen.is_empty() {
        jun.into_iter().max().unwrap() as f32
    } else {
        fn group_duplicates(mut dices: Vec<f32>) -> Vec<Vec<f32>> {
            dices.sort_unstable_by(|a, b| a.partial_cmp(b).unwrap());
            dices
                .group_by(|a, b| a == b)
                .map(|e| e.to_vec())
                .collect::<Vec<_>>()
        }

        let mut duplicates =
            group_duplicates(sen.into_iter().map(|e| e as f32).collect::<Vec<_>>());

        while duplicates.iter().any(|e| e.len() > 1) {
            let summas = duplicates
                .iter()
                .map(|e| e[0] as f32 + (e.len() / 2) as f32)
                .collect::<Vec<_>>();
            duplicates = group_duplicates(summas);
        }

        let mut results = duplicates.into_iter().flatten().collect::<Vec<_>>();
        results.sort_unstable_by(|a, b| b.partial_cmp(a).unwrap());

        let pair = results.into_iter().take(2).collect::<Vec<_>>();
        if pair.len() < 2 {
            pair[0]
        } else if pair[0] > pair[1] + 1.0 {
            pair[0]
        } else {
            pair[0] + 0.5
        }
    }
}

fn main() -> Result<(), Box<dyn Error>> {
    // let skill = 69;
    // println!("skill: {}", skill);
    // let dices_amount = get_dices_amount_by_skill(skill);
    // let mut rolled_dices = roll_dices(dices_amount);
    // rolled_dices.sort();
    // println!("rolled dices: {:?}", &rolled_dices);
    // let mut bonused_dices = add_bonuses_by_skill(rolled_dices, skill);
    // bonused_dices.sort();
    // println!("with bonuses: {:?}", &bonused_dices);
    // let total_value = calc_dices_value(&bonused_dices);
    // println!("total: {}", total_value);


    // config
    let attempts = 100_000;
    let skills = 11..=69;
    let difficulties = 5..=15;

    // generation
    let doubled_difficulties = (difficulties.start() * 2)..=(difficulties.end() * 2); // for 0.5

    let filename = PathBuf::from(format!("output/probs-{}.csv", attempts));
    if filename.exists() {
        fs::remove_file(&filename)?;
    }
    fs::create_dir_all(&filename.parent().unwrap())?;
    let file = File::create(&filename)?;

    let mut writer = csv::Writer::from_writer(file);

    writer.write_field("DIFF\\SKILL")?; // left top cell
    for skill in skills.clone() {
        writer.write_field(format!("{}", skill))?;
    }
    writer.write_record(None::<&[u8]>)?;

    for difficulty in doubled_difficulties.clone() {
        let difficulty = difficulty as f32 / 2.0;
        writer.write_field(format!("{}", difficulty))?;

        for skill in skills.clone() {
            let dices_amount = get_dices_amount_by_skill(skill);
            let dices_generator = RandomDicesGenerator::new(dices_amount);
            let generated_dices = dices_generator.take(attempts);

            let mut total = 0;
            let mut win = 0;

            for dices in generated_dices {
                let with_skill = add_bonuses_by_skill(dices, skill);
                let value = calc_dices_value(&with_skill);
                total += 1;

                if value >= difficulty {
                    win += 1;
                }
            }

            let prob = win as f64 / total as f64;
            println!(
                "probability for skill {} and difficulty {} ≈ {:.3}%",
                skill,
                difficulty,
                prob * 100.0
            );

            writer.write_field(format!("{}", ((prob * 100.0) * 100.0).floor() / 100.0))?;
        }
        writer.write_record(None::<&[u8]>)?;
    }
    writer.flush()?;

    Ok(())
}

struct RandomDicesGenerator(u32, ThreadRng);

impl RandomDicesGenerator {
    pub fn new(amount: u32) -> Self {
        Self(amount, rand::thread_rng())
    }
}

impl Iterator for RandomDicesGenerator {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        Some(
            (0..self.0)
                .map(|_| self.1.gen_range(1..=6))
                .collect::<Vec<_>>(),
        )
    }
}

struct SequinentalDicesGenerator(Vec<u32>);

impl SequinentalDicesGenerator {
    pub fn new(amount: u32) -> Self {
        let mut dices = vec![1; amount as usize];
        dices[(amount - 1) as usize] = 0;
        Self(dices)
    }
}

impl Iterator for SequinentalDicesGenerator {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        let mut last_index = self.0.len() - 1;
        self.0[last_index] += 1;
        while self.0[last_index] > 6 {
            if last_index == 0 {
                return None;
            }

            self.0[last_index] = 1;
            last_index -= 1;
            self.0[last_index] += 1;
        }
        Some(self.0.clone())
    }
}
