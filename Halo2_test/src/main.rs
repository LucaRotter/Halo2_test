use halo2_proofs::{circuit::Value, dev::MockProver};
use halo2_proofs::dev::CircuitLayout;
// Importiamo Fp solo da qui per evitare conflitti
use pasta_curves::Fp; 
mod models;
mod mathchip;
// Se MyCircuit e MyConfig sono nel file models.rs e sono pubblici (pub)
use models::MulCircuit;
use models::MuxCircuit;
use mathchip::MathCircuit_Chip;
use plotters::prelude::*;
fn main() {
    let k=4;//dimensione del circuito (2^k righe)

    //valori privati del circuito (witness)
    let a = Value::known(Fp::from(3));
    let b = Value::known(Fp::from(4));
    
    //questo lo vede il verificatore, è l'output pubblico del circuito
    let c= Fp::from(12);

    let circuit = MulCircuit { a, b, _marker: std::marker::PhantomData };
    let public_inputs = vec![vec![c]]; //input pubblico (c)
    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    
    //Il prover scorre ogni riga della tabella e controlla i tuoi polinomi.
    prover.assert_satisfied();//se il circuito è soddisfatto, non ci saranno errori. Se invece c'è un errore, verrà stampato un messaggio dettagliato con il backtrace per aiutare a identificare il problema.
    println!("Circuit is satisfied!");

    let bit = Value::known(Fp::from(0));
    let circuit_mux= MuxCircuit { a, b, bit, _marker: std::marker::PhantomData };
    let res = Fp::from(3);
    let public_inputs_mux = vec![vec![res]];
    let prover_mux = MockProver::run(k, &circuit_mux, public_inputs_mux).unwrap();
    prover_mux.assert_satisfied();
    println!("Circuit_mux is satisfied!");

    //test con chip
    let c = Value::known(Fp::from(1));
    let circuit_chip = MathCircuit_Chip { a, b, c, bit};
    let expected_res = Fp::from(7);
    let public_inputs_chip = vec![vec![expected_res]];
    let prover_chip = MockProver::run(k, &circuit_chip, public_inputs_chip).unwrap();   
    prover_chip.assert_satisfied();
    println!("Circuit_chip is satisfied!");

    let root = BitMapBackend::new("layout.png", (1024, 768)).into_drawing_area();
    root.fill(&WHITE).unwrap();
    let root = root.titled("Math Chip Layout", ("sans-serif", 60)).unwrap();

    CircuitLayout::default()
        .render(k, &circuit_chip, &root)
        .unwrap();
    
    println!("Il layout è stato salvato in layout.png");
}
