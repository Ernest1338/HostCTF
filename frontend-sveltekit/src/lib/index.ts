import { writable } from "svelte/store";

export const isLogged = writable(document.cookie.includes('logged_as'));

export async function getData(url: string): Promise<any> {
    try {
        const response = await fetch(url);
        const data = await response.json();
        return data;
    } catch (error) {
        throw new Error(`Failed to fetch data: ${error}`);
    }
}

export function getCookie(name: string): string {
    let cname = name + '=';
    let decoded_cookie = decodeURIComponent(document.cookie);
    let ca = decoded_cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(cname) == 0) {
            return c.substring(cname.length, c.length);
        }
    }
    return '';
}

export function getCookieArray(name: string): Array<string | number> {
    let cname = name + '=';
    let decoded_cookie = decodeURIComponent(document.cookie);
    let ca = decoded_cookie.split(';');
    for (let i = 0; i < ca.length; i++) {
        let c = ca[i];
        while (c.charAt(0) == ' ') {
            c = c.substring(1);
        }
        if (c.indexOf(cname) == 0) {
            return JSON.parse(c.substring(cname.length, c.length));
        }
    }
    return JSON.parse('[]');
}

export function setCookie(name: string, value: string) {
    document.cookie = name + '=' + value + ';';
}

export function setCookieArray(name: string, array: []) {
    document.cookie = name + '=' + JSON.stringify(array) + ';';
}

export function appendCookieArrayDistinct(name: string, value: string | number): boolean {
    let array = getCookieArray(name);

    if (!array.includes(value)) {
        array.push(value);
        document.cookie = name + '=' + JSON.stringify(array) + ';';
        return true;
    }

    return false;
}
