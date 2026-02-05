use crate::backup::BackupManager;
use crate::config::CloudflaredConfig;
use crate::service::ServiceManager;
use anyhow::Result;
use std::path::PathBuf;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Screen {
    Main,
    Add,
    Edit,
    Delete,
    Backup,
    Restore,
    Status,
    Help,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MainMenuItem {
    List,
    Add,
    Edit,
    Delete,
    Backup,
    Restore,
    Status,
    Help,
    Quit,
}

impl MainMenuItem {
    pub fn all() -> &'static [MainMenuItem] {
        &[
            MainMenuItem::List,
            MainMenuItem::Add,
            MainMenuItem::Edit,
            MainMenuItem::Delete,
            MainMenuItem::Backup,
            MainMenuItem::Restore,
            MainMenuItem::Status,
            MainMenuItem::Help,
            MainMenuItem::Quit,
        ]
    }

    pub fn label(&self) -> &'static str {
        match self {
            MainMenuItem::List => "List Mappings",
            MainMenuItem::Add => "Add New Mapping",
            MainMenuItem::Edit => "Edit Mapping",
            MainMenuItem::Delete => "Delete Mapping",
            MainMenuItem::Backup => "Create Backup",
            MainMenuItem::Restore => "Restore Backup",
            MainMenuItem::Status => "Service Status",
            MainMenuItem::Help => "Help",
            MainMenuItem::Quit => "Quit",
        }
    }

    pub fn shortcut(&self) -> &'static str {
        match self {
            MainMenuItem::List => "l",
            MainMenuItem::Add => "a",
            MainMenuItem::Edit => "e",
            MainMenuItem::Delete => "d",
            MainMenuItem::Backup => "b",
            MainMenuItem::Restore => "r",
            MainMenuItem::Status => "s",
            MainMenuItem::Help => "?",
            MainMenuItem::Quit => "q",
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum InputField {
    Hostname,
    Protocol,
    Host,
    Port,
    NoTlsVerify,
}

pub struct InputState {
    pub hostname: String,
    pub protocol: usize,
    pub host: String,
    pub port: String,
    pub no_tls_verify: bool,
    pub active_field: InputField,
    pub error: Option<String>,
}

impl Default for InputState {
    fn default() -> Self {
        Self {
            hostname: String::new(),
            protocol: 0,
            host: "localhost".to_string(),
            port: String::new(),
            no_tls_verify: false,
            active_field: InputField::Hostname,
            error: None,
        }
    }
}

impl InputState {
    pub fn protocols() -> &'static [&'static str] {
        &["http", "https", "ssh", "tcp"]
    }

    pub fn build_service(&self) -> String {
        let proto = Self::protocols()[self.protocol];
        format!("{}://{}:{}", proto, self.host, self.port)
    }

    pub fn validate(&self) -> Result<(), String> {
        if self.hostname.is_empty() {
            return Err("Hostname is required".to_string());
        }
        if self.port.is_empty() {
            return Err("Port is required".to_string());
        }
        if self.port.parse::<u16>().is_err() {
            return Err("Invalid port number".to_string());
        }
        Ok(())
    }

    pub fn next_field(&mut self) {
        self.active_field = match self.active_field {
            InputField::Hostname => InputField::Protocol,
            InputField::Protocol => InputField::Host,
            InputField::Host => InputField::Port,
            InputField::Port => InputField::NoTlsVerify,
            InputField::NoTlsVerify => InputField::Hostname,
        };
    }

    pub fn prev_field(&mut self) {
        self.active_field = match self.active_field {
            InputField::Hostname => InputField::NoTlsVerify,
            InputField::Protocol => InputField::Hostname,
            InputField::Host => InputField::Protocol,
            InputField::Port => InputField::Host,
            InputField::NoTlsVerify => InputField::Port,
        };
    }
}

pub struct App {
    pub config_path: PathBuf,
    pub config: CloudflaredConfig,
    pub backup_manager: BackupManager,
    pub service_manager: ServiceManager,

    pub screen: Screen,
    pub menu_index: usize,
    pub list_index: usize,
    pub backup_index: usize,

    pub input: InputState,
    pub message: Option<(String, bool)>, // (message, is_error)
    pub service_status: Option<(bool, String)>,
    pub backups: Vec<PathBuf>,

    pub should_quit: bool,
    pub confirm_action: Option<String>,
}

impl App {
    pub fn new(config_path: PathBuf) -> Result<Self> {
        let config = CloudflaredConfig::load(&config_path)?;
        let backup_manager = BackupManager::new(&config_path)?;
        let service_manager = ServiceManager::new("cloudflared");

        Ok(Self {
            config_path,
            config,
            backup_manager,
            service_manager,

            screen: Screen::Main,
            menu_index: 0,
            list_index: 0,
            backup_index: 0,

            input: InputState::default(),
            message: None,
            service_status: None,
            backups: Vec::new(),

            should_quit: false,
            confirm_action: None,
        })
    }

    pub fn reload_config(&mut self) -> Result<()> {
        self.config = CloudflaredConfig::load(&self.config_path)?;
        Ok(())
    }

    pub fn reload_backups(&mut self) -> Result<()> {
        self.backups = self.backup_manager.list_backups()?;
        Ok(())
    }

    pub fn set_message(&mut self, msg: &str, is_error: bool) {
        self.message = Some((msg.to_string(), is_error));
    }

    pub fn clear_message(&mut self) {
        self.message = None;
    }

    pub fn menu_up(&mut self) {
        if self.menu_index > 0 {
            self.menu_index -= 1;
        }
    }

    pub fn menu_down(&mut self) {
        let max = MainMenuItem::all().len() - 1;
        if self.menu_index < max {
            self.menu_index += 1;
        }
    }

    pub fn list_up(&mut self) {
        if self.list_index > 0 {
            self.list_index -= 1;
        }
    }

    pub fn list_down(&mut self) {
        let max = self.config.get_ingress_rules().len().saturating_sub(1);
        if self.list_index < max {
            self.list_index += 1;
        }
    }

    pub fn backup_up(&mut self) {
        if self.backup_index > 0 {
            self.backup_index -= 1;
        }
    }

    pub fn backup_down(&mut self) {
        let max = self.backups.len().saturating_sub(1);
        if self.backup_index < max {
            self.backup_index += 1;
        }
    }

    pub fn selected_menu_item(&self) -> MainMenuItem {
        MainMenuItem::all()[self.menu_index]
    }

    pub fn go_to_screen(&mut self, screen: Screen) {
        self.screen = screen;
        self.clear_message();
        self.input = InputState::default();

        match screen {
            Screen::Restore => {
                let _ = self.reload_backups();
                self.backup_index = 0;
            }
            Screen::Status => {
                self.service_status = self.service_manager.status().ok();
            }
            Screen::Edit | Screen::Delete => {
                self.list_index = 0;
            }
            _ => {}
        }
    }

    pub fn go_back(&mut self) {
        self.screen = Screen::Main;
        self.confirm_action = None;
        self.clear_message();
    }

    pub fn execute_add(&mut self) -> Result<()> {
        if let Err(e) = self.input.validate() {
            self.input.error = Some(e);
            return Ok(());
        }

        if self.config.hostname_exists(&self.input.hostname) {
            self.input.error = Some("Hostname already exists".to_string());
            return Ok(());
        }

        self.backup_manager.create_backup()?;

        let service = self.input.build_service();
        self.config.add_rule(
            self.input.hostname.clone(),
            service,
            self.input.no_tls_verify,
        );
        self.config.save(&self.config_path)?;

        self.set_message("Mapping added successfully", false);
        self.go_back();
        Ok(())
    }

    pub fn execute_edit(&mut self) -> Result<()> {
        if self.input.port.is_empty() || self.input.port.parse::<u16>().is_err() {
            self.input.error = Some("Invalid port".to_string());
            return Ok(());
        }

        self.backup_manager.create_backup()?;

        let service = self.input.build_service();
        self.config.update_rule(self.list_index, service);
        self.config.save(&self.config_path)?;

        self.set_message("Mapping updated successfully", false);
        self.go_back();
        Ok(())
    }

    pub fn execute_delete(&mut self) -> Result<()> {
        self.backup_manager.create_backup()?;
        self.config.remove_rule(self.list_index);
        self.config.save(&self.config_path)?;

        self.set_message("Mapping deleted successfully", false);
        self.confirm_action = None;
        self.go_back();
        Ok(())
    }

    pub fn execute_backup(&mut self) -> Result<()> {
        let path = self.backup_manager.create_backup()?;
        self.set_message(&format!("Backup created: {}", path.display()), false);
        Ok(())
    }

    pub fn execute_restore(&mut self) -> Result<()> {
        if self.backup_index < self.backups.len() {
            let backup = self.backups[self.backup_index].clone();
            self.backup_manager.restore(&backup)?;
            self.reload_config()?;
            self.set_message("Config restored successfully", false);
            self.confirm_action = None;
            self.go_back();
        }
        Ok(())
    }

    pub fn execute_restart(&mut self) -> Result<()> {
        match self.service_manager.restart() {
            Ok(msg) => self.set_message(&msg, false),
            Err(e) => self.set_message(&e.to_string(), true),
        }
        self.service_status = self.service_manager.status().ok();
        Ok(())
    }

    pub fn prepare_edit(&mut self) {
        let rules = self.config.get_ingress_rules();
        if let Some(rule) = rules.get(self.list_index) {
            self.input.hostname = rule.hostname.clone().unwrap_or_default();

            let proto = rule.get_protocol();
            self.input.protocol = InputState::protocols()
                .iter()
                .position(|&p| p == proto)
                .unwrap_or(0);

            if let Some(port) = rule.get_port() {
                self.input.port = port.to_string();
            }

            // host 파싱
            let service = &rule.service;
            if let Some(start) = service.find("://") {
                let rest = &service[start + 3..];
                if let Some(colon) = rest.rfind(':') {
                    self.input.host = rest[..colon].to_string();
                }
            }

            self.input.no_tls_verify = rule.origin_request
                .as_ref()
                .and_then(|o| o.no_tls_verify)
                .unwrap_or(false);

            self.input.active_field = InputField::Port;
        }
    }
}
