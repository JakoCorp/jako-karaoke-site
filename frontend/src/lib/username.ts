/** Matches usernames the server accepts: 1 to 64 letters, numbers, underscores and periods. */
export const USERNAME_PATTERN = /^[A-Za-z0-9_.]{1,64}$/;

/** Explains the username rules for display next to a rejected value. */
export const USERNAME_RULES_MESSAGE =
  "Usernames are 1 to 64 characters: letters, numbers, underscores and periods.";
