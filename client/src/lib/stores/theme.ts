import { atom } from 'nanostores';
import { browser } from '$app/environment';

export const darkMode = atom(false);

export function toggleDarkMode() {
	const newValue = !darkMode.get();
	darkMode.set(newValue);
	localStorage.setItem('darkMode', newValue.toString());
	updateTheme(newValue);
}

export function initTheme() {
	const storedPreference = localStorage.getItem('darkMode');
	const isDark = storedPreference
		? storedPreference === 'true'
		: window.matchMedia('(prefers-color-scheme: dark)').matches;
	darkMode.set(isDark);
	updateTheme(isDark);
}

function updateTheme(isDark) {
	if (isDark) {
		document.documentElement.classList.add('dark');
	} else {
		document.documentElement.classList.remove('dark');
	}
}

export function loadSprite() {
	if (browser) {
		const spriteLink = document.createElement('link');
		spriteLink.rel = 'preload';
		spriteLink.href = '/sprite.svg';
		spriteLink.as = 'image';
		spriteLink.type = 'image/svg+xml';
		document.head.appendChild(spriteLink);
	}
}
