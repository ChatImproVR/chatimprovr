#[derive(Default)]
struct LoginScreen {
    login_file: LoginFile,
    err_text: String,
}

impl LoginScreen {
    pub fn new(args: Opt) -> Result<Self> {
        let mut login_file = LoginFile::load()?;
        if let Some(addr) = args.connect {
            login_file.last_login_address = addr;
        }
        if let Some(user) = args.username {
            login_file.username = user;
        }

        Ok(Self {
            login_file,
            err_text: "".into(),
        })
    }

    /// Takes gl as an argument in order to create client instance (nothing else!)
    pub fn login(&mut self, gl: &Arc<gl::Context>) -> Option<Client> {
        let login_info = LoginInfo {
            username: self.login_file.username.clone(),
            address: self.login_file.last_login_address.clone(),
        };

        // Add to saved logins if not present
        if !self.login_file.addresses.contains(&login_info.address) {
            self.login_file.addresses.push(login_info.address.clone());
        }

        // Save login file
        self.login_file.addresses.sort();
        self.login_file.save().unwrap();

        log::info!(
            "Logging into {} as {}",
            login_info.address,
            login_info.username
        );
        let c = Client::new(gl.clone(), login_info);
        match c {
            Ok(c) => Some(c),
            Err(e) => {
                self.err_text = format!("Error: {:#}", e);
                None
            }
        }
    }

    /// Returns true if a login with the given login_info has been requested
    pub fn show(&mut self, ui: &mut Ui) -> bool {
        ui.label("ChatImproVR login:");

        ui.horizontal(|ui| {
            ui.label("Username: ");
            ui.text_edit_singleline(&mut self.login_file.username);
        });

        let mut ret = false;

        ui.horizontal(|ui| {
            ui.label("Address: ");
            ui.text_edit_singleline(&mut self.login_file.last_login_address);
            ret |= ui.button("Connect").clicked();
        });

        // Error text
        ui.label(RichText::new(&self.err_text).color(Color32::RED));

        ui.separator();
        ui.label("Saved logins:");

        // Login editor
        let mut dup = None;
        let mut del = None;
        for (idx, addr) in self.login_file.addresses.iter_mut().enumerate() {
            ui.horizontal(|ui| {
                ui.text_edit_singleline(addr);
                if ui.button(" + ").clicked() {
                    dup = Some(idx);
                }
                if ui.button(" - ").clicked() {
                    del = Some(idx);
                }

                if ui.button("Connect").clicked() {
                    // Move this into the address bar
                    self.login_file.last_login_address = addr.clone();
                    ret = true;
                }
            });
        }

        if let Some(del) = del {
            self.login_file.addresses.remove(del);
        }

        if let Some(dup) = dup {
            let entry = self.login_file.addresses[dup].clone();
            self.login_file.addresses.insert(dup, entry);
        }

        ret
    }
}

