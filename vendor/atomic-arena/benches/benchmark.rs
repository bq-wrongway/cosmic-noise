use atomic_arena::{Arena, Key};
use criterion::{criterion_group, criterion_main, BatchSize, BenchmarkId, Criterion};
use rand::{prelude::SliceRandom, thread_rng};

fn benchmark(c: &mut Criterion) {
	let sizes = [100, 10_000];
	for size in sizes {
		c.bench_with_input(BenchmarkId::new("reserve slots", size), &size, |b, size| {
			b.iter_batched(
				|| Arena::<()>::new(*size).controller(),
				|controller| {
					for _ in 0..*size {
						controller.try_reserve().unwrap();
					}
				},
				BatchSize::SmallInput,
			);
		});
		c.bench_with_input(BenchmarkId::new("insert", size), &size, |b, size| {
			b.iter_batched(
				|| Arena::new(*size),
				|mut arena| {
					for i in 0..*size {
						arena.insert(i).unwrap();
					}
				},
				BatchSize::SmallInput,
			);
		});
		c.bench_with_input(BenchmarkId::new("remove", size), &size, |b, size| {
			b.iter_batched(
				|| {
					let mut arena = Arena::new(*size * 2);
					let mut indices: Vec<Key> =
						(0..*size * 2).map(|i| arena.insert(i).unwrap()).collect();
					let indices_to_remove =
						Vec::from(indices.partial_shuffle(&mut thread_rng(), *size).0);
					(arena, indices_to_remove)
				},
				|(mut arena, keys_to_remove)| {
					for key in keys_to_remove {
						arena.remove(key).unwrap();
					}
				},
				BatchSize::SmallInput,
			)
		});
	}

	struct IterBenchmarkConfig {
		len: usize,
		capacity: usize,
	}
	let configs = [
		IterBenchmarkConfig {
			len: 100,
			capacity: 10_000,
		},
		IterBenchmarkConfig {
			len: 10_000,
			capacity: 10_000,
		},
	];
	for config in configs {
		c.bench_with_input(
			BenchmarkId::new("iter", format!("{} / {}", config.len, config.capacity)),
			&config,
			|b, config| {
				b.iter_batched(
					|| {
						let mut arena = Arena::new(config.capacity);
						let mut keys: Vec<Key> = (0..config.capacity)
							.map(|i| arena.insert(i).unwrap())
							.collect();
						for key in keys
							.partial_shuffle(&mut thread_rng(), config.capacity - config.len)
							.0
						{
							arena.remove(*key).unwrap();
						}
						arena
					},
					|arena| arena.iter().count(),
					BatchSize::SmallInput,
				)
			},
		);
	}
}

criterion_group!(benches, benchmark);
criterion_main!(benches);
