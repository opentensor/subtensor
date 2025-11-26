import type { Observable, OperatorFunction, UnaryFunction } from "rxjs"

/// Effects
export declare function sinkSuspense<T>(): (
  source$: Observable<T>,
) => Observable<Exclude<T, SUSPENSE>>

export function liftSuspense<T>(): (
  source$: Observable<T>,
) => Observable<T | SUSPENSE>

export declare const SUSPENSE: unique symbol
export declare type SUSPENSE = typeof SUSPENSE

/// StateObservable
export declare class StatePromise<T> extends Promise<T> {
  constructor(cb: (res: (value: T) => void, rej: any) => void)
}

// prettier-ignore
interface PipeState<T> {
  <A>(
    defaultOp: WithDefaultOperator<T, A>,
  ): DefaultedStateObservable<A>
  <A, B>(
    op1: OperatorFunction<T, A>,
    defaultOp: WithDefaultOperator<A, B>,
  ): DefaultedStateObservable<B>
  <A, B, C>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    defaultOp: WithDefaultOperator<B, C>,
  ): DefaultedStateObservable<C>
  <A, B, C, D>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    defaultOp: WithDefaultOperator<C, D>,
  ): DefaultedStateObservable<D>
  <A, B, C, D, E>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    defaultOp: WithDefaultOperator<D, E>,
  ): DefaultedStateObservable<E>
  <A, B, C, D, E, F>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    defaultOp: WithDefaultOperator<E, F>,
  ): DefaultedStateObservable<F>
  <A, B, C, D, E, F, G>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    defaultOp: WithDefaultOperator<F, G>,
  ): DefaultedStateObservable<G>
  <A, B, C, D, E, F, G, H>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
    defaultOp: WithDefaultOperator<G, H>,
  ): DefaultedStateObservable<H>
  <A, B, C, D, E, F, G, H, I>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
    op8: OperatorFunction<G, H>,
    defaultOp: WithDefaultOperator<H, I>,
  ): DefaultedStateObservable<I>
 
  (): StateObservable<T>
  <A>(
    op1: OperatorFunction<T, A>,
  ): StateObservable<A>
  <A, B>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
  ): StateObservable<B>
  <A, B, C>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
  ): StateObservable<C>
  <A, B, C, D>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
  ): StateObservable<D>
  <A, B, C, D, E>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
  ): StateObservable<E>
  <A, B, C, D, E, F>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
  ): StateObservable<F>
  <A, B, C, D, E, F, G>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
  ): StateObservable<G>
  <A, B, C, D, E, F, G, H>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
    op8: OperatorFunction<G, H>,
  ): StateObservable<H>
  <A, B, C, D, E, F, G, H, I>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
    op8: OperatorFunction<G, H>,
    op9: OperatorFunction<H, I>,
  ): StateObservable<I>
  <A, B, C, D, E, F, G, H, I>(
    op1: OperatorFunction<T, A>,
    op2: OperatorFunction<A, B>,
    op3: OperatorFunction<B, C>,
    op4: OperatorFunction<C, D>,
    op5: OperatorFunction<D, E>,
    op6: OperatorFunction<E, F>,
    op7: OperatorFunction<F, G>,
    op8: OperatorFunction<G, H>,
    op9: OperatorFunction<H, I>,
    ...operations: OperatorFunction<any, any>[],
  ): StateObservable<unknown>
}

export interface StateObservable<T> extends Observable<T> {
  getRefCount: () => number
  getValue: () => Exclude<T, SUSPENSE> | StatePromise<Exclude<T, SUSPENSE>>
  pipeState: PipeState<T>
}
export interface DefaultedStateObservable<T> extends StateObservable<T> {
  getValue: () => Exclude<T, SUSPENSE>
  getDefaultValue: () => T
}

export interface WithDefaultOperator<T, D>
  extends UnaryFunction<Observable<T>, DefaultedStateObservable<D>> {}
export declare function withDefault<T, D>(
  defaultValue: D,
): (source$: Observable<T>) => DefaultedStateObservable<T | D>

export declare class NoSubscribersError extends Error {
  constructor()
}
export declare class EmptyObservableError extends Error {
  constructor()
}

/**
 * Creates a StateObservable
 *
 * @param {Observable<T>} observable - Source observable
 * @param {T} [defaultValue] - Default value that will be used if the source
 * has not emitted.
 * @returns A StateObservable, which can be used for composing other streams that
 * depend on it. The shared subscription is closed as soon as there are no
 * subscribers, also the state is cleared.
 *
 * @remarks If the source Observable doesn't synchronously emit a value upon
 * subscription, then the state Observable will synchronously emit the
 * defaultValue if present.
 */
export declare function state<T>(
  observable: Observable<T>,
  defaultValue: T,
): DefaultedStateObservable<T>
export declare function state<T>(observable: Observable<T>): StateObservable<T>
/**
 * Creates a factory of StateObservables
 *
 * @param getObservable - Factory of Observables.
 * @param [defaultValue] - Function or value that will be used if the source
 * has not emitted.
 * @returns A function with the same parameters as the factory function, which
 * returns the StateObservable for those arguements, which can be used for
 * composing other streams that depend on it. The shared subscription is closed
 * as soon as there are no subscribers, also the state and all in memory
 * references to the returned Observable are cleared.
 *
 * @remarks If the Observable doesn't synchronously emit a value upon the first
 * subscription, then the state Observable will synchronously emit the
 * defaultValue if present.
 */
export declare function state<A extends unknown[], O>(
  getObservable: (...args: A) => Observable<O>,
  defaultValue: O | ((...args: A) => O),
): (...args: AddStopArg<A>) => DefaultedStateObservable<O>
export declare function state<A extends unknown[], O>(
  getObservable: (...args: A) => Observable<O>,
): (...args: AddStopArg<A>) => StateObservable<O>

// Adds an additional "stop" argument to prevent using factory functions
// inside high-order-functions directly (e.g. switchMap(factory$))
type AddStopArg<A extends Array<any>> = number extends A["length"]
  ? A
  : [...args: A, _stop?: undefined]
