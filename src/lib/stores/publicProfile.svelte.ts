import type { Friend } from '$lib/api/social';
let target = $state<Friend | null>(null);
let open = $state(false);
let own = $state(false);
export const publicProfile = {
    get target() { return target; }, get open() { return open; }, get own() { return own; },
    show(friend: Friend) { target = friend; own = false; open = true; },
    edit() { target = null; own = true; open = true; },
    close() { open = false; }
};
