sealed class Result<T, E> {}

class Success<T, E> extends Result<T, E> {
  final T data;

  Success(this.data);
}

class Failure<T, E> extends Result<T, E> {
  final E error;

  Failure(this.error);
}

Result<T, E> success<T, E>(T value) {
  return Success<T, E>(value);
}

Result<T, E> failure<T, E>(E error) {
  return Failure<T, E>(error);
}
