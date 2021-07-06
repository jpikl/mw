const initialTheme = localStorage.theme || 'light';
const themeSwitch = document.getElementById('theme-switch');
const themeSwitchInput = document.getElementById('theme-switch-input');

themeSwitch.removeAttribute('disabled');
themeSwitchInput.checked = initialTheme == 'dark';

themeSwitchInput.addEventListener('change', event => {
  const theme = event.target.checked ? 'dark' : 'light';
  document.documentElement.setAttribute('data-theme', theme);
  localStorage.theme = theme;
});

function focusId(id) {
  setTimeout(() => {
    document.getElementById(id).focus();
  });
}
