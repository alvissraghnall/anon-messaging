export function base64ToUrlSafe(b64: string): URLSafeBase64 {
	return b64.replace(/\+/g, '-').replace(/\//g, '_').replace(/=+$/, '');
}

export type URLSafeBase64 = string;
