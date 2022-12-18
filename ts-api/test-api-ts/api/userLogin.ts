import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export enum Error { NotAndEmail, InvalidPassword }

export type Result<T, E> = { type: "Ok", content: T } | { type: "Err", content: E };

export interface Auth { email: string, password: string, }

export interface AuthResponse { token: string, }

export function request(body: Auth): CancelablePromise<Result<AuthResponse, Error>> {
    return __request(
        { url: 'http://localhost:3000' },
        {
            method: 'GET',
            url: '/user/login',
            body,
			mediaType: 'application/json; charset=utf-8'
        }
    );
}