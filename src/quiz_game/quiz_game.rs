use std::{any::Any, collections::HashMap, fmt::Display, fs::OpenOptions, io::{stdin, Read, Write}, str::FromStr, thread, time::Duration};
use rand::{seq::IteratorRandom, thread_rng, Error};
use reqwest::get;
use serde::Deserialize;
use core::hash::Hash;

pub(crate) async fn play_quiz_game() {
    /*  
    1. ask for how many questions, and what catagory
    2. get that amount from the api (with a random gen key)
    3. parse through the info -> (put the type of trivia), (question), (save correct answer), 
    4. put all answers in random order into a random map (or whatever)
    5. print first question with type and answers ^
    6. log response (repeat 5 and 6 for all q's)
    7. for each question check right answer against log response
    8. print each check as you take it.
    */

    println!("Press q to quit at any time.");
    // category function
    let cats = trivia_req_api_categorys().await;
    println!("Please enter one of the categories of trivia to play (enter number to choose):");
    let mut cat_num: u32 = 0;
    for cat in &cats {
        cat_num+=1;
        println!("  {cat_num}. {}", cat.name);
    }
    // cat index is the index in the vector, cat id is the id number from the api
    let cat_index: usize = get_user_input().parse::<usize>().expect("your mom") - 1; // <input> - 1
    let cat_id_choosen = cats[cat_index].id; 

    println!("Please choose the dificulty in your choosen category (dif: <num of questions> is shown below):");
    let cat_dif_q_counts = req_api_question_limit(cat_id_choosen).await;
    println!("  For EASY difficulty, there are a possible {} questions", cat_dif_q_counts.total_easy_question_count);
    println!("  For MEDUIM difficulty, there are a possible {} questions", cat_dif_q_counts.total_medium_question_count);
    println!("  For HARD difficulty, there are a possible {} questions", cat_dif_q_counts.total_hard_question_count);
    println!("Enter e for easy, m for medium, or h for hard:");
    let difficulty = get_user_input().parse().expect("fadlkfj");
    let max_qs = match difficulty { 
        Difficulty::EASY => cat_dif_q_counts.total_easy_question_count,
        Difficulty::MEDUIM => cat_dif_q_counts.total_medium_question_count,
        Difficulty::HARD => cat_dif_q_counts.total_hard_question_count,
    };
    println!("Please enter the number of questions you want (maximum is {}, based on category {} and difficulty {}):", max_qs, cats[cat_index].name, difficulty);
    let num_qs: u32 = get_user_input().parse().expect("fadlkfj");
    let questions = req_api_questions(num_qs, cats[cat_index].id, difficulty).await;
    save_real_json(&questions);
    let mut answers = HashMap::new();
    let mut rng = thread_rng(); // for randomizing question order
    let mut formated_questions:Vec<String> = Vec::new(); // used for storing all the formatting logic (which is used when displaying correct answer later)
    for question in questions.iter() {
        // What is hublahbluh?  question type: General
        //  a. ()
        // ...
        // please enter your answer (a letter bozo): !!! make into own function for get letter !!!
        let mut all_answers:Vec<String> = Vec::new();
        all_answers.append(&mut question.incorrect_answers.clone());
        all_answers.push(question.correct_answer.clone());
        let mut formated_question: String = String::new();
        let question_n_type = format!("{},   question type: {}", question.question, question.r#type);
        println!("{question_n_type}");
        formated_question = question_n_type;
        for x in 0..all_answers.len() {
            let letter = match x {
                0 => 'a',
                1 => 'b',
                2 => 'c',
                3 => 'd',
                4 => 'f',
                5 => 'g',
                _ => panic!("why are there soooo many answers????"),
            };
            // get the random_q and remove it from the possible list
            let cloned_answers = all_answers.clone();
            let random_q =  cloned_answers.iter().choose(&mut rng).unwrap();
            all_answers.swap_remove(all_answers.iter().position(|s| s == random_q).unwrap());
            let one_answer = format!("  {letter}. {}", random_q);
            println!("{one_answer}");
            formated_question = formated_question + "\n" + &one_answer;
        }
        formated_questions.push(formated_question);
        println!("Please enter the letter for your answer: ");
        let answer = get_user_input();
        // save answer in answer 
        answers.insert(question, answer);
    }

    println!("HERE ARE YOUR ANSWERS!!!!");
    let mut question_num = 0;
    for answer in answers {
        println!("{}", formated_questions[question_num]);
        let mut full_answer = String::new();
        for line in formated_questions[question_num].lines() {
            if line.find(&(answer.1.to_owned() + ".")).is_some() {
                full_answer = line.trim().get(3..).expect("WTFFF").to_owned();
            }
        }
        let end_of_check_answer_msg = if answer.0.correct_answer == full_answer.trim() {
                 "... was CORRECT!!!"
        } else {
            &format!("... was incorrect :(, the correct answer was {}", answer.0.correct_answer) as &str
        };
        let checked_answer_msg = format!("Your answer of {}", answer.1) + end_of_check_answer_msg;
        println!("{checked_answer_msg}");
        question_num = question_num + 1;
    }
}

enum Difficulty {
    EASY,
    MEDUIM,
    HARD
}
// Allows for lower case formating when printing out a Difficulty enum value 
impl Display for Difficulty {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match *self {
            Self::EASY => return write!(f, "easy"),
            Self::MEDUIM => return write!(f, "meduim"),
            Self::HARD => return write!(f, "hard")
        } 
    }
}

#[derive(Debug, PartialEq, Eq)]
struct DifficultyError;

impl FromStr for Difficulty {
    type Err = DifficultyError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        if  s == "easy" || s == "e" {
            Ok(Difficulty::EASY)
        } else if s == "medium" || s == "m" {
            Ok(Difficulty::MEDUIM)
        } else if s == "hard" || s == "h" {
            Ok(Difficulty::HARD)
        } else {
            Err(DifficultyError)
        }
    }
}

#[derive(Deserialize, Debug)]
struct TriviaCategory {
    trivia_categories: Vec<Category>
}

#[derive(Deserialize, Debug)]
struct Category {
    name: String,
    id: u32
}

// SHOUDL BE SPLIT INTO ANOTHER FILE
// for getting all trivia categories available
async fn trivia_req_api_categorys() -> Vec<Category> {
    let response = reqwest::get("https://opentdb.com/api_category.php").await.expect("failed to give response");
    let categorys = response.json::<TriviaCategory>().await.expect("MASSIVE FUCKING ERROR");
    return  categorys.trivia_categories;
}

// for getting all amount questions available for a given category from the api 
#[derive(Deserialize, Debug)]
struct CatQsInfo {
    category_id: u32,
    category_question_count: CatQsCount
}
#[derive(Deserialize, Debug)]
struct CatQsCount {
    total_question_count: u32,
    total_easy_question_count: u32,
    total_medium_question_count: u32,
    total_hard_question_count: u32
}
async fn req_api_question_limit(cat_id: u32) -> CatQsCount {
    let response = reqwest::get(format!("https://opentdb.com/api_count.php?category={cat_id}")).await.expect("failed to give response");
    let categorys = response.json::<CatQsInfo>().await.expect("MASSIVE FUCKING ERROR");
    return  categorys.category_question_count;
}

#[derive(Deserialize, Debug)]
struct QuestionAPIResponse {
    response_code: u32,
    results: Vec<MultiChoiceQuestionInfo>
}

#[derive(Deserialize, Debug)]
struct MultiChoiceQuestionInfo {
    r#type: String,
    difficulty: String,
    category: String,
    question: String,
    correct_answer: String,
    incorrect_answers: Vec<String>
}


impl PartialEq for MultiChoiceQuestionInfo {
    fn eq(&self, other: &Self) -> bool {
        self.r#type == other.r#type && self.difficulty == other.difficulty && self.category == other.category && self.question == other.question && self.correct_answer == other.correct_answer && self.incorrect_answers == other.incorrect_answers
    }
}

impl Hash for MultiChoiceQuestionInfo {
    fn hash<H: std::hash::Hasher>(&self, state: &mut H) {
        self.r#type.hash(state);
        self.difficulty.hash(state);
        self.category.hash(state);
        self.question.hash(state);
        self.correct_answer.hash(state);
        self.incorrect_answers.hash(state);
    }
}

impl  Eq for MultiChoiceQuestionInfo {
    
}
async fn req_api_questions(amount: u32, category: u32, difficulty: Difficulty) -> Vec<MultiChoiceQuestionInfo> {

    // This section allows for the user to play a game of trivia with over 50 
    let mut mutable_amount = amount;
    let mut all_questions: Vec<MultiChoiceQuestionInfo> = Vec::new();
    loop {
        if mutable_amount > 50 {thread::sleep(Duration::from_secs(5));} else {break;}
        mutable_amount = mutable_amount - 50;
        let response = reqwest::get(format!("https://opentdb.com/api.php?amount=50&category={category}&difficulty={difficulty}&type=multiple")).await.expect("failed to give response");
        let mut formatted_response = response.json::<QuestionAPIResponse>().await.expect("MASSIVE FUCKING ERROR");  
        all_questions.append(&mut formatted_response.results);
    }


    let response = reqwest::get(format!("https://opentdb.com/api.php?amount={amount}&category={category}&difficulty={difficulty}&type=multiple")).await.expect("failed to give response");
    let mut formatted_response = response.json::<QuestionAPIResponse>().await.expect("MASSIVE FUCKING ERROR");
    all_questions.append(&mut formatted_response.results);
    save_real_json(&all_questions);
    return  all_questions;
}

fn save_real_json(mult_choice_questions: &Vec<MultiChoiceQuestionInfo>) {
    let mut debug_file = OpenOptions::new().append(true).open("/Users/wyattbracy/Desktop/Rust101/first_rust_proj/src/quiz_game/local_trivia/debug.txt").expect("Failed opening debug file");
    let mut to_write_str = String::new();
    for question in mult_choice_questions {
        to_write_str = to_write_str + &format!("question: {}\n", question.question);
        to_write_str = to_write_str + &format!("correct answer: {}\n", question.correct_answer);
        to_write_str = to_write_str + &format!("wrong answers: ");
        for wrong_answer in question.incorrect_answers.iter() {
            to_write_str = to_write_str + &format!("{}, ", wrong_answer)
        }
        to_write_str = to_write_str + "\n";
    }
    debug_file.write_all(to_write_str.as_bytes()).expect("your mo");
}

fn get_user_input() -> String {
    let mut user_input = String::new();
    stdin().read_line(&mut user_input).expect("failed to read input");
    return user_input.trim().to_owned();
}