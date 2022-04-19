use anyhow::{Context, Result};
use cfg_if::cfg_if;
use log::info;

use crate::components::runtime::Runtime;
use crate::connector::request::{
    CreateProfile,
    GetConfig,
    GetProfileList,
    GetSiteList,
    GetSystemVersions,
    InstallRuntime,
    InstallSite,
    LaunchSite,
    RemoveProfile,
    SetConfig,
    UninstallRuntime,
    UninstallSite,
    UpdateAllSites,
    UpdateProfile,
    UpdateSite,
};
use crate::connector::response::ConnectorResponse;
use crate::connector::Connection;
use crate::console::app::{
    ProfileCreateCommand,
    ProfileRemoveCommand,
    ProfileUpdateCommand,
    RuntimeInstallCommand,
    RuntimeUninstallCommand,
    SiteInstallCommand,
    SiteLaunchCommand,
    SiteUninstallCommand,
    SiteUpdateCommand,
};
use crate::console::Run;
use crate::storage::Storage;

pub trait Process {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse>;
}

impl Process for GetSystemVersions {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        Ok(ConnectorResponse::SystemVersions {
            firefoxpwa: Some(env!("CARGO_PKG_VERSION").into()),
            firefox: Runtime::new(connection.dirs)?.version,
            _7zip: {
                cfg_if! {
                    if #[cfg(target_os = "windows")] {
                        use crate::components::_7zip::_7Zip;
                        _7Zip::new()?.version
                    } else {
                        None
                    }
                }
            },
        })
    }
}

impl Process for GetConfig {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        let storage = Storage::load(connection.dirs)?;
        Ok(ConnectorResponse::Config(storage.config))
    }
}

impl Process for SetConfig {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        let mut storage = Storage::load(connection.dirs)?;
        storage.config = self.0.to_owned();
        storage.write(connection.dirs)?;
        Ok(ConnectorResponse::ConfigSet)
    }
}

impl Process for InstallRuntime {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = RuntimeInstallCommand {};
        command.run()?;

        Ok(ConnectorResponse::RuntimeInstalled)
    }
}

impl Process for UninstallRuntime {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = RuntimeUninstallCommand {};
        command.run()?;

        Ok(ConnectorResponse::RuntimeUninstalled)
    }
}

impl Process for GetSiteList {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        let storage = Storage::load(connection.dirs)?;
        Ok(ConnectorResponse::SiteList(storage.sites))
    }
}

impl Process for LaunchSite {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        cfg_if! {
            if #[cfg(target_os = "macos")] { let command = SiteLaunchCommand { id: self.id, url: self.url.to_owned(), arguments: vec![], direct_launch: false }; }
            else { let command = SiteLaunchCommand { id: self.id, url: self.url.to_owned(), arguments: vec![] }; }
        };
        command.run()?;

        Ok(ConnectorResponse::SiteLaunched)
    }
}

impl Process for InstallSite {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = SiteInstallCommand {
            manifest_url: self.manifest_url.to_owned(),
            document_url: self.document_url.to_owned(),
            start_url: self.start_url.to_owned(),
            profile: self.profile.to_owned(),
            name: self.name.to_owned(),
            description: self.description.to_owned(),
            categories: self.categories.to_vec(),
            keywords: self.keywords.to_vec(),
            system_integration: true,
        };
        let ulid = command._run()?;

        Ok(ConnectorResponse::SiteInstalled(ulid))
    }
}

impl Process for UninstallSite {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = SiteUninstallCommand { id: self.id, quiet: true, system_integration: true };
        command.run()?;

        Ok(ConnectorResponse::SiteUninstalled)
    }
}

impl Process for UpdateSite {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = SiteUpdateCommand {
            id: self.id,
            start_url: self.start_url.to_owned(),
            name: self.name.to_owned(),
            description: self.description.to_owned(),
            categories: self.categories.to_vec(),
            keywords: self.keywords.to_vec(),
            update_manifest: self.update_manifest,
            update_icons: self.update_icons,
            system_integration: true,
            store_none_values: true,
        };
        command.run()?;

        Ok(ConnectorResponse::SiteUpdated)
    }
}

impl Process for UpdateAllSites {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        let mut storage = Storage::load(connection.dirs)?;

        for site in storage.sites.values_mut() {
            info!("Updating web app {}", site.ulid);

            if self.update_manifest {
                site.update().context("Failed to update web app manifest")?;
            }

            site.update_system_integration(connection.dirs)
                .context("Failed to update system integration")?;
        }

        storage.write(connection.dirs)?;
        Ok(ConnectorResponse::AllSitesUpdated)
    }
}

impl Process for GetProfileList {
    fn process(&self, connection: &Connection) -> Result<ConnectorResponse> {
        let storage = Storage::load(connection.dirs)?;
        Ok(ConnectorResponse::ProfileList(storage.profiles))
    }
}

impl Process for CreateProfile {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = ProfileCreateCommand {
            name: self.name.to_owned(),
            description: self.description.to_owned(),
        };
        let ulid = command._run()?;

        Ok(ConnectorResponse::ProfileCreated(ulid))
    }
}

impl Process for RemoveProfile {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = ProfileRemoveCommand { id: self.id, quiet: true };
        command.run()?;

        Ok(ConnectorResponse::ProfileRemoved)
    }
}

impl Process for UpdateProfile {
    fn process(&self, _connection: &Connection) -> Result<ConnectorResponse> {
        let command = ProfileUpdateCommand {
            id: self.id,
            name: self.name.to_owned(),
            description: self.description.to_owned(),
            store_none_values: true,
        };
        command.run()?;

        Ok(ConnectorResponse::ProfileUpdated)
    }
}