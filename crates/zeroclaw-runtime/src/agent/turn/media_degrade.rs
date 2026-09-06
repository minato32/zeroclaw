//! Failed-turn media degradation, shared by the ACP restore and live-session paths.

use zeroclaw_api::model_provider::ConversationMessage;
use zeroclaw_providers::multimodal;

/// Replace image references in this message span with an omission note.
///
/// A turn that ended in a non-retryable failure may have had its attachments
/// rejected by the provider. That rejection is durable knowledge about those
/// attachments: resending them on the next request reproduces the same
/// rejection, so one bad attachment would poison every later prompt on the
/// session. The marker is replaced with a note rather than deleted so the
/// model still sees that something was attached; surrounding text survives.
///
/// Returns the number of image references degraded. Callers own the span
/// policy (which messages belong to the failed turn); this helper only
/// performs the projection.
pub fn degrade_media_in_messages(messages: &mut [ConversationMessage]) -> usize {
    let omitted = crate::i18n::get_required_cli_string("turn-failed-attachment-omitted");
    let mut degraded = 0;
    for message in messages {
        let ConversationMessage::Chat(chat) = message else {
            continue;
        };
        let (cleaned, refs) = multimodal::parse_image_markers(&chat.content);
        if refs.is_empty() {
            continue;
        }
        degraded += refs.len();
        chat.content = if cleaned.is_empty() {
            omitted.clone()
        } else {
            format!("{cleaned}\n\n{omitted}")
        };
    }
    degraded
}
