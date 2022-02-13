export function getDefferedPromise<T>(): [
    Promise<T>,
    (value: T | PromiseLike<T>) => void,
    (reason: any) => void
] {
    let resolve: (value: T | PromiseLike<T>) => void;
    let reject: (reason: any) => void;

    const promise = new Promise<T>((res, rej) => {
        resolve = res;
        reject = rej;
    });

    return [promise, resolve!, reject!]
}