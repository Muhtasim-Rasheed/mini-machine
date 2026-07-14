pub trait Machine {
    type Inst: std::fmt::Debug;
    fn decode(&self, ip: &mut usize) -> Result<Self::Inst, String>;
    fn execute(&mut self, inst: Self::Inst, ip: &mut usize) -> bool;
    fn debug_dump(&self, old_ip: usize) -> String;
    fn format_ip(&self, ip: usize) -> String {
        ip.to_string()
    }

    fn run(mut self, debug: bool)
    where
        Self: Sized,
    {
        let mut ip = 0;
        loop {
            let old_ip = ip;
            let inst = match self.decode(&mut ip) {
                Ok(i) => i,
                Err(e) => {
                    eprintln!("IP {}: {e}", self.format_ip(old_ip));
                    std::process::exit(1);
                }
            };
            if debug {
                println!("[Executing] IP {}: {inst:?}", self.format_ip(old_ip));
            }
            let halted = self.execute(inst, &mut ip);
            if debug {
                println!("{}", self.debug_dump(old_ip));
                if !halted {
                    let mut line = String::new();
                    std::io::stdin().read_line(&mut line).unwrap();
                    if line.trim().eq_ignore_ascii_case("q") {
                        return;
                    }
                }
            }
            if halted {
                return;
            }
        }
    }
}
