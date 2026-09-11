export interface Account {
	id: string;
	username: string;
	uuid: string;
	minecraftToken: string;
	expiresAt: number;
}

function createAccountStore() {
	let value = $state<Account | null>(null);
	return {
		get value() {
			return value;
		},
		set value(a: Account | null) {
			value = a;
		},
		clear() {
			value = null;
		}
	};
}

export const account = createAccountStore();
