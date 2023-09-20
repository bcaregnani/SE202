

use clap::Parser;

#[derive(Parser)]
#[command( author = "Bruno Pons", about = "Compute Fibonacci suite values", long_about = None)]
struct Args {

    #[arg(short, long, default_value_t = false,  help="Print intermediate values")]
    verbose: bool,

    // The minimum number to compute
    #[arg(value_name="NUMBER" ,short, long, default_value_t = 0, help="The minimum number to compute")]
    min: u32,

    value: u32,
}

/// Recursive fibo
// fn fibo(n: u32) -> u32 {
//     if n == 0 {
//         0
//     } else if n == 1 {
//         1
//     } else {
//         fibo(n-1) + fibo(n-2)
//     }
// }


/// Iterative fibo
// fn fibo(n: u32) -> u32 {
//     let mut a: u32 = 0;
//     let mut b: u32 = 1;
//     let mut c: u32;
//     if n == 0 {
//         return a
//     } else if n == 1 {
//         return b
//     } else {
//         for _ in 2..=n {
//             // c = a.saturating_add(b); //Arithmétique saturée
//             c = a.checked_add(b).unwrap(); // Arithmétique vérifiée
//             a = b;
//             b = c;
//         };
//     };
//     b
// }


/// Display only correct values
fn fibo(n: u32) -> Option<u32> {
    let mut a: Option<u32> = Some(0);
    let mut b: Option<u32> = Some(1);
    let mut c: Option<u32>;
    if n == 0 {
        return a
    } else if n == 1 {
        return b
    } else {
        for _ in 2..=n {
            c = a.unwrap().checked_add(b.unwrap());
            a = b;
            b = c;
        };
    };
    b
}





fn main() {
    let args = Args::parse();
    let mut i = args.min;
    let mut cond = true;
    while (i<=args.value) & cond {

        match fibo(i) {
            Some(x) => if args.verbose | (i==args.value) {println!("Number {} => Some({})", i, x)},
            None         => if args.verbose | (i==args.value) {
                println!("None");
                cond = false;
            },
        };
        i += 1;
    
    };
}
