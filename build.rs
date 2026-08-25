// Faz o cargo relinkar quando o kernel.ld mudar.
//
// Sem isto o cargo so olha os .rs: tu edita o linker script, ele responde
// "Finished" sem fazer nada, e tu fica depurando o binario antigo.
fn main() {
    println!("cargo::rerun-if-changed=kernel.ld");
}
