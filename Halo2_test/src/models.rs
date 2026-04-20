use halo2_proofs::{circuit::*, plonk::*, poly::Rotation};
use std::marker::PhantomData;

//definizione delle colonne
#[derive(Clone, Debug)]
struct MyConfig{
    a: Column<Advice>,
    b: Column<Advice>,
    c: Column<Instance>,
    s: Selector,
}

//definizione del circuito
struct MyCircuit<F>{
    a: Value<F>,
    b: Value<F>,
    _marker: PhantomData<F>,
}

impl<F: Field> Circuit<F> for MyCircuit<F>{
    type Config = MyConfig;
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

        MyConfig { a, b, c, s }
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