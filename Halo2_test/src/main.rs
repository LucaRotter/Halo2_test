use halo2_proofs::{circuit::Value, dev::MockProver};
// Importiamo Fp solo da qui per evitare conflitti
use pasta_curves::Fp; 
mod models;
// Se MyCircuit e MyConfig sono nel file models.rs e sono pubblici (pub)
use models::MyCircuit;

fn main() {
    let k=4;//dimensione del circuito (2^k righe)

    //valori privati del circuito (witness)
    let a = Value::known(Fp::from(3));
    let b = Value::known(Fp::from(4));
    
    //questo lo vede il verificatore, è l'output pubblico del circuito
    let c= Fp::from(12);

    let circuit = MyCircuit { a, b, _marker: std::marker::PhantomData };


    let public_inputs = vec![vec![c]]; //input pubblico (c)

    let prover = MockProver::run(k, &circuit, public_inputs).unwrap();
    
    //Il prover scorre ogni riga della tabella e controlla i tuoi polinomi.
    prover.assert_satisfied();//se il circuito è soddisfatto, non ci saranno errori. Se invece c'è un errore, verrà stampato un messaggio dettagliato con il backtrace per aiutare a identificare il problema.
    println!("Circuit is satisfied!");
}
