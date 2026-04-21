use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;
use halo2_proofs::arithmetic::Field;
use ff::PrimeField;


//definizione delle colonne
#[derive(Clone, Debug)]
pub struct MulConfig{
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Instance>,
    s: Selector,
}
#[derive(Default)]
//definizione del circuito
pub struct MulCircuit<F>{
    pub(crate) a: Value<F>,
    pub(crate) b: Value<F>,
    pub(crate) _marker: PhantomData<F>,
}

impl<F: Field> Circuit<F> for MulCircuit<F>{
    type Config = MulConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    //defizione dei gates 
    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();//colonna per i dati privati
        let b = meta.advice_column();//sempre per dati privati
        let c = meta.instance_column();//colonna per i dati pubblici
        let s = meta.selector();//selettore per attivare il gate/vincoli

        //definiamo la nostra prima (e unica) gate
        meta.create_gate("a * b = c", |meta| {//|meta| è una closure che prende in input un oggetto meta che ci permette di accedere ai valori delle colonne e dei selettori
            let s = meta.query_selector(s);
            let a = meta.query_advice(a, Rotation::cur());//Rotation::cur() indica che stiamo prendendo il valore della colonna a nella stessa riga del selettore
            let b = meta.query_advice(b, Rotation::cur());
            let c = meta.query_instance(c, Rotation::cur());

            // REGOLA: s * (a * b - c) deve essere SEMPRE 0
            // Se s è 0 (OFF), l'equazione è 0=0 (sempre vera, nessun vincolo)
            // Se s è 1 (ON), allora a * b deve essere uguale a c
            // Il vincolo matematico: s * (a * b - c) = 0
            vec![s * (a * b - c)]
        });

        MulConfig { a, b, c, s }
    }

    fn synthesize(&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error> {
        //il layouter è l'oggetto che ci permette di assegnare i valori alle colonne e attivare i selettori
        layouter.assign_region(
            || "moltiplicazione",
            |mut region| {
                config.s.enable(&mut region, 0)?;//attiviamo il selettore per questa riga
                region.assign_advice(|| "a", config.a, 0, || self.a)?;
                region.assign_advice(|| "b", config.b, 0, || self.b)?;
                Ok(())
            },
        )
    }
}
//circuito if-else
#[derive(Clone, Debug)]
pub struct MuxConfig{
    pub a: Column<Advice>,
    pub b: Column<Advice>,
    pub bit: Column<Advice>,
    pub res: Column<Instance>,
    pub s: Selector,
}

#[derive(Default)]
pub struct MuxCircuit<F> {
    pub a: Value<F>,
    pub b: Value<F>,
    pub bit: Value<F>,
    pub _marker: PhantomData<F>,
}

impl<F: PrimeField> Circuit<F> for MuxCircuit<F> {
    type Config = MuxConfig;
    type FloorPlanner = SimpleFloorPlanner;

    fn without_witnesses(&self) -> Self {
        Self::default()
    }

    fn configure(meta: &mut ConstraintSystem<F>) -> Self::Config {
        let a = meta.advice_column();
        let b = meta.advice_column();
        let bit = meta.advice_column();
        let res = meta.instance_column();
        let s = meta.selector();

        meta.create_gate("mux", |meta| {
            let s = meta.query_selector(s);
            let a = meta.query_advice(a, Rotation::cur());
            let b = meta.query_advice(b, Rotation::cur());
            let bit = meta.query_advice(bit, Rotation::cur());
            let res = meta.query_instance(res, Rotation::cur());

            let one = Expression::Constant(F::from(1u64));
            //vincolo bit solo 1 o 0: bit*(1-bit)
            let bool_check = bit.clone()*(one.clone() - bit.clone());
            //F::one() crea la costante 1
            //Expression:: Costant(...) Halo dice di trattare questo numero come una costante nell'espressione

            //formula MUX: res = (1 - bit)* a + (bit * b)
            //non si può usare = quindi si sposta tutto da una parte
            //bit=1 è b, bit=0 è a
            let mux_logic = (one -bit.clone())*a + (bit * b)-res;

            vec![s.clone() * bool_check, s * mux_logic]
        });

        MuxConfig { a, b, bit, res, s }
    }
    fn synthesize (&self, config: Self::Config, mut layouter: impl Layouter<F>) -> Result<(), Error>{
          layouter.assign_region(
            || "if-else",
            |mut region| {
                config.s.enable(&mut region, 0)?;//attiviamo il selettore per questa riga
                region.assign_advice(|| "a", config.a, 0, || self.a)?;
                region.assign_advice(|| "b", config.b, 0, || self.b)?;
                region.assign_advice(|| "bit", config.bit, 0, || self.bit)?;
                Ok(())
            },
        )
    }
}