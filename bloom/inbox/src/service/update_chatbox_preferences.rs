use super::{DetailedChatboxPreferences, UpdateChatboxPreferencesInput};
use crate::{Error, Service};
use kernel::{consts::BillingPlan, Actor};
use stdx::chrono::Utc;

impl Service {
    pub async fn update_chatbox_preferences(
        &self,
        actor: Actor,
        input: UpdateChatboxPreferencesInput,
    ) -> Result<DetailedChatboxPreferences, kernel::Error> {
        let actor = self.kernel_service.current_user(actor)?;

        let (namespace, _) = self
            .kernel_service
            .find_namespace_and_membership(&self.db, actor.id, input.namespace_id)
            .await?;

        let mut preferences = self
            .repo
            .find_chatbox_preferences_for_namespace(&self.db, input.namespace_id)
            .await?;

        let name = input.name.trim().to_string();
        self.validate_chatbox_name(&name)?;

        let color = input.color.trim().to_string();
        self.validate_chatbox_color(&color)?;

        let welcome_message = input.welcome_message.trim().to_string();
        self.validate_chatbox_welcome_message(&welcome_message)?;

        let show_branding = input.show_branding;
        if !show_branding {
            if !self.kernel_service.self_hosted() && namespace.plan == BillingPlan::Free {
                return Err(Error::UpgradePlanToRemoveChatboxBranding.into());
            }
        }

        preferences.updated_at = Utc::now();
        preferences.name = name;
        preferences.color = color;
        preferences.welcome_message = welcome_message;
        preferences.show_branding = show_branding;
        self.repo.update_chatbox_preferences(&self.db, &preferences).await?;

        let base_url = self.kernel_service.base_url();

        let ret = DetailedChatboxPreferences {
            preferences,
            base_url,
        };

        Ok(ret)

        // ret = support.ChatboxPreferencesAndProjectPublicData{
        //     ChatboxPreferences: preferences,
        //     AvatarURL:          service.kernelService.ProjectAvatarURL(project.Avatar),
        //     TwitterURL:         project.TwitterURL,
        //     FacebookURL:        project.FacebookURL,
        //     PublicEmail:        project.PublicEmail,
        //     InstagramURL:       project.InstagramURL,
        //     WhatsappNumber:     project.WhatsappNumber,
        //     MastodonURL:        project.MastodonURL,
        //     HomepageURL:        project.HomepageURL,
        // }
    }
}
