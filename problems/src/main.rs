use std::ops::{Deref, DerefMut};



// The generic parameter " 'a " represents the lifetime of the reference
enum OOR<'a> {
    Owned(String),
    Borrowed(&'a str),
}


impl<'a> Deref for OOR<'a>
{
    type Target = str;

    fn deref(&self) -> &Self::Target
    {
        match self {
            OOR::Owned(s) => s.as_str(),
            OOR::Borrowed(s) => s,
        }
    }
}
// The lifetime of the resulting &str is that of the variable it references

// This is always appropiate because: we can't have shorter lifetime because 
// we would lose reference of the variable, we can't outlive the referenced
// variable.



impl<'a> DerefMut for OOR<'a>
{
    fn deref_mut(&mut self) -> &mut Self::Target
    {
        match self
        {
            OOR::Owned(s) => s.as_mut_str(),
            OOR::Borrowed(s) => 
            {
                *self = OOR::Owned(String::from(*s));
                match self
                {
                    OOR::Owned(s) => s.as_mut_str(),
                    // this next part of the code is unreachable but it is necessary 
                    // to add it so that the match is complete
                    OOR::Borrowed(_) => unreachable!(),
                }
            },
        }
    }
}




fn ret_string() -> String {
    String::from("  A String object  ")
}

fn choose_str<'c, 'a: 'c, 'b: 'c>(s1: &'a str, s2: &'b str, select_s1: bool) -> &'c str {
    if select_s1 { s1 } else { s2 }
}

fn main() {

    let s = ret_string();
    assert_eq!(s.trim(), "A String object");


    // BEGIN MY TEST

    let aux = OOR::Owned( String::from("Hello World") );
    assert_eq!(aux.trim(), "Hello World");

    let aux2 = OOR::Borrowed(" Hello world ");
    assert_eq!(aux2.trim(), "Hello world");

    // END MY TEST




    // Check Deref for both variants of OOR
    let s1 = OOR::Owned(String::from("  Hello, world.  "));
    assert_eq!(s1.trim(), "Hello, world.");
    let mut s2 = OOR::Borrowed("  Hello, world!  ");
    assert_eq!(s2.trim(), "Hello, world!");

    // Check choose
    let s = choose_str(&s1, &s2, true);
    assert_eq!(s.trim(), "Hello, world.");
    let s = choose_str(&s1, &s2, false);
    assert_eq!(s.trim(), "Hello, world!");

    // Check DerefMut, a borrowed string should become owned
    assert!(matches!(s1, OOR::Owned(_)));
    assert!(matches!(s2, OOR::Borrowed(_)));
    unsafe {
        for c in s2.as_bytes_mut() {
            if *c == b'!' {
                *c = b'?';
            }
        }
    }
    assert!(matches!(s2, OOR::Owned(_)));
    assert_eq!(s2.trim(), "Hello, world?");

}

/*
 * 
 * Q: Why the code fails to compile? Ask yourself: what is the lifetime of s? 
 *    Who is the owner of the underlying string with spaces (every object has an owner)?
 * A: The code fails to compile because "trim()" is borrowing the expression "ret_string()" 
 *    as this is a call to a function it creates a temporary value that stores the return 
 *    value of the function so that it can be borrowed. As this is a temporary value the end 
 *    of its lifetime is after the let, so when we try to use the variable in the "assert_eq!()" 
 *    this variable is not available anymore.
 * 
 * 
 * 
 */