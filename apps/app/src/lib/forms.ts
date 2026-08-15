// Shared vee-validate field behavior: don't nag while the user is still
// typing. Fields validate on blur; once a field has an error, it re-validates
// on every keystroke so the message clears the moment the input is fixed.
//
// Usage: const [email, emailProps] = defineField("email", lazyThenEager);
export function lazyThenEager(state: { errors: string[] }): {
  validateOnModelUpdate: boolean;
} {
  return { validateOnModelUpdate: state.errors.length > 0 };
}
