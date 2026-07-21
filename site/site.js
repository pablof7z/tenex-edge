const setupPrompt =
  'Go to https://mosaico.f7z.io/SETUP.md and follow the instructions.';

const button = document.querySelector('[data-copy-setup]');
const status = document.querySelector('[data-copy-status]');

button?.addEventListener('click', async () => {
  button.disabled = true;
  status.textContent = 'Copying setup prompt...';

  try {
    await navigator.clipboard.writeText(setupPrompt);
    button.textContent = 'Copied';
    status.textContent = 'Paste the prompt into your coding agent.';
  } catch {
    button.textContent = 'Copy setup prompt';
    status.textContent = `Copy this: ${setupPrompt}`;
  } finally {
    button.disabled = false;
  }
});
