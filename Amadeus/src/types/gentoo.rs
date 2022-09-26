#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Bug {
  pub assigned_to: String,
  pub creation_time: String,
  pub creator: String,
  pub is_open: bool,
  pub priority: String,
  pub product: String,
  pub severity: String,
  pub resolution: String,
  pub status: String,
  pub summary: String,
}

#[allow(non_snake_case)]
#[derive(Deserialize, Debug)]
pub struct Bugs {
  pub bugs: Vec<Bug>
}

pub type Wiki = (String, Vec<String>, Vec<String>, Vec<String>);

//["Portage",["Portage","Portage-Multi-Stage-Dockerfile","Portage-utils","Portage Multi Stage Dockerfile"
//,"Portage Prefix Python Venv Usr Local Multi Distro","Portage Security","Portage TMPDIR on tmpfs"
//,"Portage log","Portage log/de","Portage log/en"],["","","","","","","","","",""]
//,["https://wiki.gentoo.org/wiki/Portage","https://wiki.gentoo.org/wiki/Portage-Multi-Stage-Dockerfile"
//,"https://wiki.gentoo.org/wiki/Portage-utils"
//,"https://wiki.gentoo.org/wiki/Portage_Multi_Stage_Dockerfile"
//,"https://wiki.gentoo.org/wiki/Portage_Prefix_Python_Venv_Usr_Local_Multi_Distro"
//,"https://wiki.gentoo.org/wiki/Portage_Security","https://wiki.gentoo.org/wiki/Portage_TMPDIR_on_tmpfs"
//,"https://wiki.gentoo.org/wiki/Portage_log","https://wiki.gentoo.org/wiki/Portage_log/de"
//,"https://wiki.gentoo.org/wiki/Portage_log/en"]
//]
//["Gay",[],[],[]]
