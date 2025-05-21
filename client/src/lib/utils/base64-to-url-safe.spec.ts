import { describe, it, expect } from 'vitest';
import { base64ToUrlSafe } from './base64-to-url-safe';

describe('base64ToUrlSafe', () => {
	it('should replace + with -', () => {
		expect(base64ToUrlSafe('a+b=')).toBe('a-b');
	});

	it('should replace / with _', () => {
		expect(base64ToUrlSafe('a/b=')).toBe('a_b');
	});

	it('should remove padding =', () => {
		expect(base64ToUrlSafe('abcd==')).toBe('abcd');
	});

	it('should handle combined replacements and padding', () => {
		expect(base64ToUrlSafe('a+/b==')).toBe('a-_b');
	});

	it('should leave valid URL-safe Base64 unchanged', () => {
		expect(base64ToUrlSafe('a-_b')).toBe('a-_b');
	});
});
