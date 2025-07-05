#![no_std]

use soroban_sdk::{
    contract, contracterror, contractimpl, contracttype, panic_with_error, symbol_short, token,
    xdr::ToXdr, Address, Bytes, BytesN, Env, IntoVal, Map, Symbol, Val,
};

// --- Estructura de Datos para el Depósito ---
// Almacena la información clave de cada fideicomiso.
#[contracttype]
#[derive(Clone)]
pub struct Deposit {
    pub depositor: Address,
    pub beneficiary: Address,
    pub token: Address,
    pub amount: i128,
    pub released: bool,
}

// --- Errores Personalizados del Contrato ---
// Define los posibles errores que el contrato puede devolver.
#[contracterror]
#[derive(Copy, Clone, Debug, Eq, PartialEq, PartialOrd, Ord)]
#[repr(u32)]
pub enum Error {
    AlreadyInitialized = 1, // El contrato ya fue inicializado.
    NotInitialized = 2,     // El contrato no ha sido inicializado.
    Unauthorized = 3,       // La acción no está autorizada.
    InvalidAmount = 4,      // El monto debe ser mayor que cero.
    InvalidBeneficiary = 5, // El beneficiario no puede ser el mismo que el depositante.
    DepositExists = 6,      // Ya existe un depósito con los mismos parámetros.
    DepositNotFound = 7,    // El depósito especificado no se encontró.
    AlreadyReleased = 8,    // Los fondos del depósito ya fueron liberados o reembolsados.
}

// --- Claves para el Almacenamiento ---
// Símbolos cortos y eficientes para acceder a los datos en el storage.
const OWNER: Symbol = symbol_short!("owner");
const DEPOSITS: Symbol = symbol_short!("deposits");

#[contract]
pub struct SecureEscrow;

#[contractimpl]
impl SecureEscrow {
    /// --- INICIALIZACIÓN ---
    /// Inicializa el contrato estableciendo un propietario.
    /// Solo se puede llamar una vez.
    pub fn initialize(env: Env, owner: Address) {
        if env.storage().instance().has(&OWNER) {
            panic_with_error!(&env, Error::AlreadyInitialized);
        }
        env.storage().instance().set(&OWNER, &owner);
    }

    fn generate_deposit_id(
        env: &Env, // Es buena práctica pasar el Env como referencia
        caller: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        nonce: BytesN<32>,
    ) -> BytesN<32> {
        // Paso 1: Convertir la tupla de datos a un 'Val' genérico.
        // Esto agrupa todos los datos en un solo tipo de Soroban.
        let data_val: Val = (
            caller.clone(),
            beneficiary.clone(),
            token.clone(),
            amount,
            nonce.clone(),
        )
            .into_val(env);

        // Paso 2 (LA SOLUCIÓN): Serializar explícitamente el 'Val' a 'Bytes' usando XDR.
        // La función de hash necesita una secuencia de bytes concreta, no un tipo abstracto.
        let data_bytes: Bytes = data_val.to_xdr(env);

        // Paso 3: Ahora sí, podemos hashear el objeto 'Bytes' resultante.
        env.crypto().sha256(&data_bytes).into()
    }

    /// --- DEPÓSITO ---
    /// Un usuario deposita tokens en el contrato para un beneficiario.
    pub fn deposit(
        env: Env,
        caller: Address,
        beneficiary: Address,
        token: Address,
        amount: i128,
        nonce: BytesN<32>,
    ) {
        caller.require_auth();
        if amount <= 0 {
            panic_with_error!(&env, Error::InvalidAmount);
        }
        if beneficiary == caller {
            panic_with_error!(&env, Error::InvalidBeneficiary);
        }

        let deposit_id = Self::generate_deposit_id(
            &env,
            caller.clone(),
            beneficiary.clone(),
            token.clone(),
            amount,
            nonce.clone(),
        );

        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| Map::new(&env));

        if deposits.contains_key(deposit_id.clone()) {
            panic_with_error!(&env, Error::DepositExists);
        }

        let deposit_data = Deposit {
            depositor: caller.clone(),
            beneficiary: beneficiary.clone(),
            token: token.clone(),
            amount,
            released: false,
        };

        deposits.set(deposit_id.clone(), deposit_data);
        env.storage().persistent().set(&DEPOSITS, &deposits);

        let token_client = token::Client::new(&env, &token);
        let contract_address = env.current_contract_address();

        // CORRECCIÓN 2: La función `transfer` en esta versión del SDK espera `MuxedAddress`.
        // Se debe realizar la conversión de `Address` a `MuxedAddress` con `.into()`.
        token_client.transfer(
            &caller.clone().into(),
            &contract_address.clone().into(),
            &amount,
        );

        env.events().publish(
            (symbol_short!("deposit"), caller, beneficiary),
            (deposit_id, amount),
        );
    }

    /// --- LIBERACIÓN DE FONDOS ---
    /// El propietario del contrato libera los fondos al beneficiario.
    pub fn release_funds(env: Env, deposit_id: BytesN<32>) {
        let owner: Address = env
            .storage()
            .instance()
            .get(&OWNER)
            .unwrap_or_else(|| panic_with_error!(&env, Error::NotInitialized));
        owner.require_auth();

        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        let mut deposit = deposits
            .get(deposit_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        if deposit.released {
            panic_with_error!(&env, Error::AlreadyReleased);
        }

        deposit.released = true;
        deposits.set(deposit_id.clone(), deposit.clone());
        env.storage().persistent().set(&DEPOSITS, &deposits);

        let token_client = token::Client::new(&env, &deposit.token);

        // CORRECCIÓN 3: Convertir los `Address` a `MuxedAddress` para la función `transfer`.
        token_client.transfer(
            &env.current_contract_address().clone().into(),
            &deposit.beneficiary.clone().into(),
            &deposit.amount,
        );

        env.events().publish(
            (symbol_short!("release"), deposit.beneficiary),
            (deposit_id, deposit.amount),
        );
    }

    /// --- REEMBOLSO ---
    /// El depositante original solicita un reembolso.
    pub fn refund(env: Env, deposit_id: BytesN<32>) {
        let mut deposits = env
            .storage()
            .persistent()
            .get::<_, Map<BytesN<32>, Deposit>>(&DEPOSITS)
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        let mut deposit = deposits
            .get(deposit_id.clone())
            .unwrap_or_else(|| panic_with_error!(&env, Error::DepositNotFound));

        deposit.depositor.require_auth();

        if deposit.released {
            panic_with_error!(&env, Error::AlreadyReleased);
        }

        deposit.released = true;
        deposits.set(deposit_id.clone(), deposit.clone());
        env.storage().persistent().set(&DEPOSITS, &deposits);

        let token_client = token::Client::new(&env, &deposit.token);

        // CORRECCIÓN 4: Convertir los `Address` a `MuxedAddress` para la función `transfer`.
        token_client.transfer(
            &env.current_contract_address().clone().into(),
            &deposit.depositor.clone().into(),
            &deposit.amount,
        );

        env.events().publish(
            (symbol_short!("refund"), deposit.depositor),
            (deposit_id, deposit.amount),
        );
    }
}
