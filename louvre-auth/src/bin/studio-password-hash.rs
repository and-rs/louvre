fn main() {
    let password =
        rpassword::prompt_password("Studio password: ").expect("failed to read studio password");
    let confirmation = rpassword::prompt_password("Confirm studio password: ")
        .expect("failed to read password confirmation");

    if password.is_empty() || password != confirmation {
        eprintln!("passwords must be non-empty and match");
        std::process::exit(1);
    }

    println!("{}", louvre_auth::generate_password_hash(password));
}
