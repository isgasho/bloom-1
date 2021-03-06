<template>
  <v-container fill-height fluid class="pa-0">

    <v-row justify="center" class="ma-0 pa-0">
      <v-col cols="4" lg="3" class="pa-0 bloom-left-col">
        <div class="overflow-y-auto b-conversations-list">
          <v-list-item-group
            mandatory
            v-model="selectedConversationIndex"
            @change="selectedConversationIndexChanged"
          >
            <v-list two-line class="pa-0">
              <template v-for="(conversation, index) in conversations" class="bloom-pointer">

                <v-list-item :key="`conversation-${index}`">

                  <v-list-item-avatar>
                    <v-icon dark>
                      mdi-account
                    </v-icon>
                  </v-list-item-avatar>

                  <v-list-item-content class="text-left">
                    <v-list-item-title>
                      {{ conversation.conversation.name }}
                    </v-list-item-title>
                    <v-list-item-subtitle>
                      {{ conversation.messages[conversation.messages.length - 1].body_html }}
                    </v-list-item-subtitle>
                  </v-list-item-content>

                  <v-list-item-action>
                    <v-list-item-action-text>
                      {{ calendar(conversation.conversation.last_message_at) }}
                    </v-list-item-action-text>
                  </v-list-item-action>

                </v-list-item>
                <v-divider v-if="index !== conversations.length - 1" :key="index"/>

              </template>
            </v-list>
          </v-list-item-group>
        </div>
      </v-col>


      <v-col cols="8" lg="9" class="pa-0">
        <v-app-bar
          dense
          color="white"
          class="elevation-0 contact-appbar"
          v-if="selectedConversation">
          <!-- <v-avatar size="40">
            <img
              :src="selectedConversation.contact.avatarUrl"
            />
          </v-avatar> -->

          <v-toolbar-title>
        <!-- <router-link :to="`/${projectFullPath}/-/contacts/${selectedConversation.contact.id}`">
            {{ selectedConversation.contact.name }}
            </router-link> -->
            {{ selectedConversation.conversation.name }}
          </v-toolbar-title>
        </v-app-bar>

        <v-alert icon="mdi-alert-circle" type="error" :value="error !== ''" v-if="error !== ''">
          {{ error }}
        </v-alert>

        <div class="conversation overflow-y-auto" ref="conversation" v-if="selectedConversation">
          <v-progress-circular
            v-if="loading"
            :size="50"
            color="primary"
            indeterminate
          />

          <template v-for="(message, i) in messages" v-else>
            <b-message :message="message" :key="i" />
          </template>
        </div>
        <div v-else-if="!selectedConversation && chatboxPreferences">
          <b-chatbox-setup-card class="ma-5 pa-5" :preferences="chatboxPreferences" />
        </div>

        <v-textarea
          class="conversation-input pa-0"
          placeholder="Compose your message..."
          v-model="message"
          hide-details
          @keydown="onInputKeyDown"
          v-if="selectedConversation"
          :loading="loadingSend"
        />
      </v-col>
    </v-row>

  </v-container>
</template>


<script lang="ts">
import { VueApp } from '@/app/vue';
import BMessage from '@/ui/components/inbox/message.vue';
import { calendar } from '@/app/filters';
import { InboxSubscriptionOptions } from '@/domain/inbox/service';
import {
  ChatboxPreferences,
  ConversationWithContactsAndMessages, Message, SendMessage,
} from '@/domain/inbox/model';
import BChatboxSetupCard from '@/ui/components/inbox/chatbox_setup_card.vue';

export default VueApp.extend({
  name: 'BInboxView',
  components: {
    BMessage,
    BChatboxSetupCard,
  },
  data() {
    return {
      loading: false,
      loadingSend: false,
      error: '',
      selected: 'All',
      selectedConversationIndex: 0,
      selectedConversation: null as ConversationWithContactsAndMessages | null,
      message: '',
      messages: [] as Message[],
      conversations: [] as ConversationWithContactsAndMessages[],
      seenMessages: new Set<string>(),
      seenConversations: new Set<string>(),
      baseUrl: '',
      chatboxPreferences: null as ChatboxPreferences | null,

    };
  },
  computed: {
    projectFullPath(): string {
      return `${this.$route.params.namespacePath}/${this.$route.params.projectPath}`;
    },
  },
  watch: {
    messages() {
      setTimeout(this.scrollToBottom, 50);
    },
  },
  created() {
    this.fetchData();
    this.subscribeToMessages();
  },
  beforeDestroy() {
    this.$inboxService.unsubscribeFromInbox();
  },
  methods: {
    calendar,
    async fetchData() {
      this.loading = true;
      this.error = '';

      try {
        const [inbox, chatboxPreferences] = await Promise.all([
          this.$inboxService.fetchInbox(),
          this.$inboxService.fetchChatboxPreferences(),
        ]);

        this.chatboxPreferences = chatboxPreferences;
        this.conversations = inbox.conversations;

        this.conversations.forEach((conversation) => {
          this.seenConversations.add(conversation.conversation.id);
          // eslint-disable-next-line max-len
          conversation.messages.forEach((message) => this.seenMessages.add(message.id));
        });

        if (this.conversations.length !== 0) {
          this.messages = this.conversations[this.selectedConversationIndex].messages;
          this.selectedConversation = this.conversations[this.selectedConversationIndex];
        }
        this.loading = false;
        VueApp.nextTick(() => {
          this.scrollToBottom();
        });
      } catch (err) {
        this.error = err.message;
      }
    },
    subscribeToMessages() {
      const options: InboxSubscriptionOptions = {
        onData: this.onConversation,
        onError: console.error,
      };
      this.$inboxService.subscribeToInbox(options);
    },
    async sendMessage() {
      if (this.message.trim() === '') {
        return;
      }

      this.loadingSend = true;
      this.error = '';
      const input: SendMessage = {
        conversation_id: this.conversations[this.selectedConversationIndex].conversation.id,
        body: this.message,
      };

      try {
        const message = await this.$inboxService.sendMessage(input);
        this.onMessage(message);
        this.message = '';
      } catch (err) {
        this.error = err.message;
      } finally {
        this.loadingSend = false;
      }
    },
    onConversation(conversation: ConversationWithContactsAndMessages): void {
      if (!this.seenConversations.has(conversation.conversation.id)) {
        // new conversation
        conversation.messages.forEach((message) => this.seenMessages.add(message.id));
        const index = this.conversations.length >= 1 ? 1 : 0;
        this.conversations.splice(index, 0, conversation);
        this.seenConversations.add(conversation.conversation.id);
      } else {
        // existing conversation
        conversation.messages.forEach((message) => {
          this.onMessage(message);
        });
      }

      if (this.conversations.length === 1) {
        this.selectedConversationIndexChanged(0);
      }
    },
    onMessage(message: Message): void {
      if (!this.seenMessages.has(message.id)) {
        this.conversations.forEach((conversation) => {
          if (conversation.conversation.id === message.conversation_id) {
            conversation.messages.push(message);
          }
        });
        this.seenMessages.add(message.id);
      }
    },
    selectedConversationIndexChanged(selected: number | undefined) {
      if (!selected) {
        this.selectedConversationIndex = 0;
      } else {
        this.selectedConversationIndex = selected;
      }
      this.selectedConversation = this.conversations[this.selectedConversationIndex];
      this.messages = this.selectedConversation.messages;
    },
    scrollToBottom() {
      const container = this.$refs.conversation;
      if (container) {
        // eslint-disable-next-line @typescript-eslint/no-explicit-any
        (container as any).scrollTop = (container as any).scrollHeight;
      }
    },
    onInputKeyDown(e: KeyboardEvent) {
      if (e.keyCode === 13 && !e.shiftKey) {
        e.preventDefault();
        this.sendMessage();
      }
    },
  },
});
</script>

<style lang="scss" scoped>
.v-toolbar {
  border-bottom: 1px solid rgba($color: #000000, $alpha: 0.1) !important;
  left: 0px !important;
}

.bloom-left-col {
  border-right: 1px solid #dedede;
}

.conversation {
  height: calc(100vh - 244px);
  display: flex;
  flex-direction: column;
  flex: 1;
  background-color: #f4f7fd;
  padding: 0 18px 0 8px;
}

.contact-appbar {
  background-color: #fff;
  .v-toolbar__title {
    margin-left: 10px;
  }
}

.b-conversations-list {
  height: calc(100vh - 50px);
}
</style>
