import { request as __request } from '../request';
import { CancelablePromise } from '../CancelablePromise';

export function request(body: string): CancelablePromise<number> {
    return __request(
        { url: 'http://localhost:3000' },
        {
            method: 'GET',
            url: '/user',
            body,
			mediaType: 'application/json; charset=utf-8'
        }
    );
}