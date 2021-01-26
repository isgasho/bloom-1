/* eslint-disable */
const prefix = 'inbox';
const commands = 'commands';
const queries = 'queries';

export const Commands = {
  createContact: `/${prefix}/${commands}/create_contact`,
  deleteContact: `/${prefix}/${commands}/delete_contact`,
  importContacts: `/${prefix}/${commands}/import_contacts`,
  sendMessage: `/${prefix}/${commands}/send_message`,
  updateChetboxPreferences:  `/${prefix}/${commands}/update_chatbox_preferences`,
  updateContact:  `/${prefix}/${commands}/update_contact`,
}

export const Queries = {
  archive: `/${prefix}/${queries}/archive`,
  chatboxPreferences: `/${prefix}/${queries}/chatbox_preferences`,
  contact: `/${prefix}/${queries}/contact`,
  contacts: `/${prefix}/${queries}/contacts`,
  inbox: `/${prefix}/${queries}/inbox`,
  newsletterList: `/${prefix}/${queries}/newsletter_list`,
  newsletterLists: `/${prefix}/${queries}/newsletter_lists`,
  spam: `/${prefix}/${queries}/spam`,
  trash: `/${prefix}/${queries}/trash`,
}