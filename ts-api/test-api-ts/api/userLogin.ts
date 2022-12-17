import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export interface Auth { email: string, password: string, }

export interface AuthResponse { token: string, }

export type Result<T, E> = { type: "Ok", content: T} | { type: "Err", content: E };

export function request(json: Auth): CancelablePromise<Result<AuthResponse, number>> {
    return __request(
        { url: 'http://localhost:3000' },
        {
            method: 'GET',
            url: '/user/login',
            body: JSON.stringify(json)
        }
    );
}