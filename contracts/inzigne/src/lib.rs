use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, token,
    Address, BytesN, Env, Symbol, Vec,
};

use soroban_env_common::env::Env;

// --- ESTRUCTURA DE DATOS PARA LOS DEPÓSITOS ---
// Es una buena práctica definir una estructura para los datos complejos.
#[contracttype]
#[derive(Clone)]
pub struct Deposit {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address, // Token del depósito (ej. XLM o USDC)
    pub amount: i128,
    pub released: bool,
}

// --- ERRORES PERSONALIZADOS ---
// CORRECCIÓN: Usar errores de contrato en lugar de panics con unwrap().
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1,
    NotInitialized = 2,
    Unauthorized = 3,
    InvalidAmount = 4,
    InvalidBeneficiary = 5,
    DepositExists = 6,
    DepositNotFound = 7,
    AlreadyReleased = 8,
}

// --- LLAVES PARA EL ALMACENAMIENTO ---
// CORRECCIÓN: Usar constantes para las llaves es más eficiente y seguro.
const OWNER: Symbol = symbol_short!("owner");
const DEPOSITS: Symbol = symbol_short!("deposits");

#[contract]
pub struct SecureEscrow;

#[contractimpl]
impl SecureEscrow {
    /// Inicializa el contrato estableciendo el owner. Solo se puede llamar una vez.
    pub fn initialize(env: Env, owner: Address) {
        // CORRECCIÓN: Evitar que el contrato se reinicialice.
        if env.storage().instance().has(&OWNER) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&OWNER, &owner);
    }

    pub fn deposit(
        env: Env,
        beneficiary: Address,
        token: Address,
        amount: i128,
        nonce: BytesN<32>,
    ) {
        // CORRECCIÓN 1: Usar require_auth() para obtener y autorizar al depositante.
        let depositor: Address = env.get_invoker();
        depositor.require_auth();

        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        if beneficiary == depositor {
            panic_with_error!(&env, Error::InvalidBeneficiary);
        }

        let deposit_id: BytesN<32> = env
            .crypto()
            .sha256(
                &(
                    depositor.clone(),
                    beneficiary.clone(),
                    token.clone(),
                    amount,
                    nonce,
                )
                    .into_val(&env),
            )
            .into(); // No olvides el .into() que corregimos antes

        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, soroban_sdk::Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| soroban_sdk::Map::new(&env));

        if deposits.contains_key(deposit_id.clone()) {
            panic_with_error!(&env, Error::DepositExists);
        }

        let deposit_data = Deposit {
            depositor: depositor.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount,
            released: false,
        };

        deposits.set(deposit_id.clone(), deposit_data);
        env.storage().persistent().set(&DEPOSITS, &deposits);

        // CORRECCIÓN 2: Asegúrate de que token_client se define aquí
        // y de que transfer_from tiene 4 argumentos.
        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();
        token_client.transfer_from(&contract_address, &depositor, &contract_address, &amount);

        env.events().publish(
            (symbol_short!("deposit"), depositor, beneficiary),
            (deposit_id, amount),
        );
    }

    /// Liberar fondos al beneficiario. Solo el owner puede hacerlo.
    pub fn release_funds(env: Env, deposit_id: BytesN<32>) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&OWNER)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));
        owner.require_auth(); // Solo el owner puede liberar.

        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, soroban_sdk::Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        let mut deposit = deposits
            .get(deposit_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        if deposit.released {
            panic_with_error!(&env, Error::AlreadyReleased);
        }

        // Marcar como liberado y actualizar el estado
        deposit.released = true;
        deposits.set(deposit_id.clone(), deposit.clone());
        env.storage().persistent().set(&DEPOSITS, &deposits);

        // Transferir los fondos al beneficiario
        let token_client = token::Client::new(&env, &deposit.token);
        token_client.transfer(
            &env.current_contract_address(),
            &deposit.beneficiary,
            &deposit.amount,
        );

        env.events().publish(
            (symbol_short!("release"), deposit.beneficiary),
            (deposit_id, deposit.amount),
        );
    }

    /// Reembolsar fondos al depositante si no han sido liberados.
    pub fn refund(env: Env, deposit_id: BytesN<32>) {
        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, soroban_sdk::Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        let mut deposit = deposits
            .get(deposit_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        // CORRECCIÓN: Solo el depositante puede pedir el reembolso.
        deposit.depositor.require_auth();

        if deposit.released {
            panic_with_error!(&env, Error::AlreadyReleased);
        }

        // Marcar como liberado (para evitar doble gasto) y actualizar
        deposit.released = true;
        deposits.set(deposit_id.clone(), deposit.clone());
        env.storage().persistent().set(&DEPOSITS, &deposits);

        // Devolver los fondos al depositante
        let token_client = token::Client::new(&env, &deposit.token);
        token_client.transfer(
            &env.current_contract_address(),
            &deposit.depositor,
            &deposit.amount,
        );

        env.events().publish(
            (symbol_short!("refund"), deposit.depositor),
            (deposit_id, deposit.amount),
        );
    }
}
