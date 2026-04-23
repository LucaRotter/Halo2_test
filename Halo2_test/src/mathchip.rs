use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;
use halo2_proofs::arithmetic::Field;
use ff::PrimeField;

#[derive(Clone, Debug)]
pub struct MathConfig {
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub out: Column<Advice>,//risultato intermedio
    pub res: Column<Instance>,//risultato finale
    pub s_mul: Selector,
    pub s_mux: Selector,
    pub s_add: Selector,
    pub bit : Column<Advice>,//colonna per il bit di controllo del mux
}

pub struct MathChip<F: PrimeField> {
    config: MathConfig,
    _marker: PhantomData<F>,
}

impl<F: PrimeField> MathChip<F> {
    pub fn construct(config: MathConfig) -> Self {
        Self { config, _marker: PhantomData }
    }

    pub fn configure(meta: &mut ConstraintSystem<F>) -> MathConfig {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let out = meta.advice_column();
        let bit = meta.advice_column();
        let res = meta.instance_column();
        
        let s_mul = meta.selector();
        let s_add = meta.selector();
        let s_mux = meta.selector();

        // GATE MOLTIPLICAZIONE
        meta.create_gate("mul", |meta| {
            let s = meta.query_selector(s_mul);
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());
            vec![s * (lhs * rhs - out)]
        });

        // GATE ADDIZIONE
        meta.create_gate("add", |meta| {
            let s = meta.query_selector(s_add);
            let lhs = meta.query_advice(a, Rotation::cur());
            let rhs = meta.query_advice(b, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());
            vec![s * (lhs + rhs - out)]
        });

        // GATE MUX (If-Else)
        meta.create_gate("mux", |meta| {
            let s = meta.query_selector(s_mux);
            let val_a = meta.query_advice(a, Rotation::cur());
            let val_b = meta.query_advice(b, Rotation::cur());
            let b_ctrl = meta.query_advice(bit, Rotation::cur());
            let out = meta.query_advice(out, Rotation::cur());
            let one = Expression::Constant(F::from(1u64));

            vec![
                s.clone() * b_ctrl.clone() * (one.clone() - b_ctrl.clone()), // bool check
                s * ((one - b_ctrl.clone()) * val_a + (b_ctrl * val_b) - out) // mux logic
            ]
        });

        MathConfig { a, b, out, res, s_mul, s_add, s_mux, bit }
    }

    pub fn do_mul(&self, mut layouter: impl Layouter<F>, a: Value<F>, b: Value<F>) -> Result<(), Error> {
        layouter.assign_region(|| "mul operation", |mut region| {// || è una closure che prende in input un regione restituisce un Result
            self.config.s_mul.enable(&mut region, 0)?;
            region.assign_advice(|| "a", self.config.a, 0, || a)?;
            region.assign_advice(|| "b", self.config.b, 0, || b)?;
            // Calcoliamo il risultato per l'advice 'out'
            let res = a.zip(b).map(|(a, b)| a * b);
            region.assign_advice(|| "out", self.config.out, 0, || res)?;
            Ok(())
        })
    }

    pub fn do_add(&self, mut layouter: impl Layouter<F>, a: Value<F>, b: Value<F>) -> Result<AssignedCell<F, F>, Error> {
        layouter.assign_region(|| "add operation", |mut region| {
            self.config.s_add.enable(&mut region, 0)?;
            region.assign_advice(|| "a", self.config.a, 0, || a)?;
            region.assign_advice(|| "b", self.config.b, 0, || b)?;
            let res = a.zip(b).map(|(a, b)| a + b);
            region.assign_advice(|| "out", self.config.out, 0, || res)
        })
    }

    pub fn do_mux(&self, mut layouter: impl Layouter<F>, a: Value<F>, b: Value<F>, bit: Value<F>) -> Result<(), Error> {
        layouter.assign_region(|| "mux operation", |mut region| {
            self.config.s_mux.enable(&mut region, 0)?;
            region.assign_advice(|| "a", self.config.a, 0, || a)?;
            region.assign_advice(|| "b", self.config.b, 0, || b)?;
            region.assign_advice(|| "bit", self.config.bit, 0, || bit)?;
            // Calcoliamo il risultato per l'advice 'out'
            let res = a.zip(b).zip(bit).map(|((a, b), bit)| if bit == F::from(1u64) { b } else { a });
            region.assign_advice(|| "out", self.config.out, 0, || res)?;
            Ok(())
        })
    }

    

}

//circuito che utilizza il chip
#[derive(Default)]
pub struct MathCircuit_Chip<F> {
    pub a: Value<F>,
    pub b: Value<F>,
    pub c: Value<F>,
    pub bit: Value<F>,
}

impl<F: PrimeField> Circuit<F> for MathCircuit_Chip<F> {
    type Config = MathConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
       MathChip::configure(meta)
    }
    
    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        let chip = MathChip::construct(config);
        // 1. Esegui addizione: a + b
        let add_cell = chip.do_add(layouter.namespace(|| "step 1"), self.a, self.b)?;

        // 2. Esegui moltiplicazione usando il risultato dell'addizione: (a+b) * c
        chip.do_mul(layouter.namespace(|| "step 2"), add_cell.value().cloned(), self.c)?;

        // 3. Esegui MUX: sceglie tra (a+b) e c basandosi sul bit
        chip.do_mux(layouter.namespace(|| "step 3"), add_cell.value().cloned(), self.c, self.bit)?;

        Ok(())
    }
}