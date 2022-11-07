use std::collections::HashMap; 
use std::fs::{File, OpenOptions}; 
use std::io::{Read, Write}; 
use std::path::PathBuf; 
use structopt::StructOpt; 
use thiserror::Error;

//food
#[derive(Debug)]
struct Food {
    id: i64,
    name: String,
    stock: i64,
    price: i64,
}

#[derive(Debug)]
struct Foods {
    inner: HashMap<i64, Food>
}
impl Foods { 
    fn new() -> Self { 
        Self { inner: HashMap::new(), }
    }

    fn next_id(&self) -> i64 {
        let mut ids: Vec<_> = self.inner.keys().collect();
        ids.sort();
        match ids.pop() {
            Some(id) => id + 1,
            None => 1
        }
    }

    fn add(&mut self, food: Food) {
        self.inner.insert(food.id, food);
    }

    fn into_vec(mut self) -> Vec<Food> {
        let mut foods: Vec<_> = self.inner.drain().map(|kv| kv.1).collect();
        foods.sort_by_key(|fd| fd.id);
        foods
    }


    fn remove(&mut self, name: &str) -> bool {
        let temp: HashMap<_, _> = self.inner.clone().drain().filter(|kv| kv.1.name != name ).collect();

        if temp.len() == self.inner.len() {
            return false
        }

        self.inner = temp;
        true
    }

    fn sell(mut self, food: Food) -> i64 {
        let temp: Vec<_> = self.inner.clone().drain().filter(|kv| kv.1.name == food.name ).collect();

        let new = parse_food(&format!("{},{},{},{}", temp.first().unwrap().1.id, 
        temp.first().unwrap().1.name, 
        temp.first().unwrap().1.stocks - food.stocks, 
        temp.first().unwrap().1.price)
        );

        self.add(new.unwrap());

        match save_foods(self){
            Ok(()) => (),
            Err(_e) => println!("Save foods error...")
        };

        temp.first().unwrap().1.price * (food.stocks as i64)
    }

    fn insert(&mut self, mut food: Food) {
        let new: HashMap<_, _> = self.inner.clone().drain().filter(|kv| kv.1.name != food.name).collect();

        if new.len() != self.inner.len() { 
            let food_data: Vec<_> = self.foods.clone().drain().filter(|kv| kv.1.name == food.name).collect();
            food.id = food_data.get(0).unwrap().0;
            food.price = food_data.get(0).unwrap().1.price;
            food.stocks += &food_data.get(0).unwrap().1.stocks;
            println!("Berhasil mengubah data makanan, {}, dengan stock {}, dan harga Rp {},00", food.name, food.stocks, food.price)
        }else { 
            food.id = self.next_id();
            println!("Berhasil menambahkan makanan baru, {}, dengan stock {}, dan harga Rp {},00", food.name, food.stocks, food.price);
        }

        self.add(food);
    }

    pub fn is_empty_food(self) -> bool {
        self.foods.is_empty()

}

//Parse err
#[derive(Debug, Error)]
enum ParseError {
    #[error("id must be a number: {0}")]
    InvalidId(#[from] std::num::ParseIntError),
    #[error("empty food")]
    EmptyRecord,
    #[error("missing field: {0}")]
    MissingField(String),
}

fn parse_food(food: &str) -> Result<Record, ParseError> {
    let fields: Vec<&str> = food.split(',').collect();

    let id = match strings.first() {
        Some(id) => id.parse::<u64>()?,
        None => return Err(error::ParseError::InvalidInput("id - foods"))
    };

    let name = match strings.get(1).filter(|name| !name.is_empty()) {
        Some(name) => name.to_string(),
        None => return Err(error::ParseError::MissingField("name"))
    };

    let stocks = match strings.get(2) {
        Some(stocks) => stocks.parse::<u64>()?,
        None => return Err(error::ParseError::InvalidInput("stocks"))
    };

    let price = match strings.get(3) {
        Some(price) => price.trim().parse::<u128>()?,
        None => return Err(error::ParseError::InvalidInput("price"))
    };

    Ok(Food { id, name, stocks, price})

}

fn parse_foods(foods: String, verbose: bool) -> Foods {
    let mut foods = Records::new();
    for (num, food) in foods.split('\n').enumerate() {
        if food != "" {
            match parse_food(food) {
                Ok(fd) => foods.add(fd),
                Err(e) => {
                    if verbose {
                        println!(
                            "error on line number {}: {}\n > \"{}\"\n", 
                            num + 1, 
                            e, 
                            food
                        )
                    }
                }
            }
        }
    }
    foods
}

fn load_records(file_name: PathBuf, verbose: bool) -> std::io::Result<Records> {
    let mut file = std::fs::File::open(PathBuf::from("bin/food.csv"))?;
    let mut buffer = String::new();
    file.read_to_string(&mut buffer)?;

    Ok(parsing_record(buffer, verbose))
}

fn save_records(file_name: PathBuf, foods: Records) -> std::io::Result<()> {
    let mut file = std::fs::OpenOptions::new().write(true).truncate(true).open(PathBuf::from("bin/food.csv"))?;


    file.write_all( b"id,name,stock,price\n")?;

    file.flush()?;

    for food in foods.into_vec().into_iter() {
        if food.stocks.eq(&0) {
            continue;
        }
        file.write_all(format!("{},{},{},{}\n", food.id, food.name, food.stocks, food.price).as_bytes())?;
    }

    file.flush()?;

    Ok(())
}

//opt
#[derive(Debug, StructOpt)]
#[structopt(about = "Dev Restaurant")]
struct Opt {
    #[structopt(subcommand)]
    cmd: Command,
    #[structopt(short, help = "verbose")]
    verbose: bool
}

#[derive(Debug, StructOpt)]
enum Command {
    List,
    Report,
    Buy {
        name: String,
        stocks: i64
    },
    Add {
        name: String,
        stocks: i64,
        price: i64
    },
    Delete {
        name: String
    }
}
impl Opt {
    fn run(opt: Opt) -> Result<(), std::io::Error> {
        match opt.cmd {
            Command::Add { name, stocks, price } => {
                let mut foods = foods::load_foods(opt.verbose)?;

                match foods::parse_food(&format!("0,{name},{stocks},{price}")) {
                    Ok(food) => {
                        foods.insert(food);
                        foods::save_foods(foods)?;
                    },
                    Err(e) => println!("{:?}", e)
                }

                Ok(())
            }

            Command::List => {
                let foods = foods::load_foods(opt.verbose)?;
                for food in foods.into_vec() {
                    println!("{:?}", food)
                }
                Ok(())
            }
            Command::Delete { name } => {
                let mut foods = foods::load_foods(opt.verbose)?;

                if foods.remove(&name) { // True == Success
                    foods::save_foods(foods)?;
                    println!("Berhasil menghapus {name} dari daftar");
                }else {
                    println!("Tidak ada makanan dengan nama {name} di data");
                };

                Ok(())
            }
            Command::Buy { name, stocks } => {
                let mut reports = reports::load_reports(opt.verbose)?;
                let foods = foods::load_foods(opt.verbose)?;

                let (income, stock) = match foods::parse_food(&format!("0,{name},{stocks},0")) {
                    Ok(food) => foods.sell(food),
                    Err(_e) => (0, 0)
                };
        }
        Ok(())
    }
    
}



fn main() {
    let opt = Opt::from_args();
    if let Err(e) = run(opt) {
        println!("an error occured: {}", e);
    }
}