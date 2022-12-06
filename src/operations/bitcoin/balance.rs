use std:rc:Rc;

use anyhow::Result;

use bdk::{blockchain::esplora::EsploraBlockchain,data::MemoryData, SyncOption, Segwit, Taproot}

use crate::{
   data::


pub trait ExtScriptContext: ScriptContext {
    /// Returns the [`ScriptContext`] as a [`ScriptContextEnum`]
    fn as_enum() -> ScriptContextEnum;

    }

    /// Returns whether the script context is [`Segwitv0`](miniscript::Segwitv0)
    fn is_segwit_v0() -> bool {
        Self::as_enum().is_segwit_v0()
    }

    /// Returns whether the script context is [`Tap`](miniscript::Tap), aka Taproot or Segwit V1
    fn is_taproot() -> bool {
        Self::as_enum().is_taproot()
    }
