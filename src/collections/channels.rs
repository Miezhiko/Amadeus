lazy_static! {

/*
 * Channels where I can spam (well, chat actually)
 */
pub static ref AI_ALLOWED: Vec<String>  = dhall!("dhall/channels/ai_allowed.dhall");

/*
 * Channels where I can learn
 */
pub static ref AI_LEARN: Vec<String>    = dhall!("dhall/channels/ai_learn.dhall");

}
