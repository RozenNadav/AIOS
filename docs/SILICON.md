# AIOS Silicon Lattice

AIOS does **not** expose GPUs or CPUs as Unix `/dev` nodes or Windows WDDM devices.  
Hardware is a first-class cognition substrate: the **Silicon Lattice**.

## Why

Classic OSes treat NVIDIA GPUs and AMD chips as driver afterthoughts — brittle stacks (WDDM, NVIDIA proprietary + Wayland fights, ROCm install hell). AIOS binds compute as **lanes** that agents request through intents, with vessel policy and Clarity grants.

## Primitives

| Term | Meaning | Not the same as |
|------|---------|-----------------|
| **Die** | A physical/logical silicon unit (CPU package or GPU) | PCI device / `/dev/nvidia0` |
| **Lane** | A compute pathway on a die (CUDA, Tensor, ROCm, Zen cores) | driver API handle |
| **Lattice** | The set of discovered dies + lanes | device manager |
| **Bind** | Attach an agent to a lane via facet | CUDA context / HIP context |
| **Accel** | Run a named compute burst on a bound lane | kernel launch |

## Supported vendors (DevCore 0.2+)

### NVIDIA GPUs
- Probe: `nvidia-smi`, NVML-style presence, PCI class display controllers (10de)
- Lanes: `cuda`, `tensor`, `rt` (ray), `encode`
- Facets: `silicon.nvidia.cuda`, `silicon.nvidia.tensor`

### AMD chips
- **CPU (Zen and family):** probe via CPUID vendor `AuthenticAMD`, core/thread topology
- **GPU (RDNA/CDNA):** probe ROCm/`rocminfo`, PCI vendor `1002`
- Lanes: `zen` (CPU), `rocm`, `rdna`, `encode`
- Facets: `silicon.amd.zen`, `silicon.amd.rocm`

## Intents

```
silicon probe                 # discover lattice
silicon list                  # show dies + lanes
silicon bind <agent> <die>    # grant lane facets to agent
silicon unbind <agent>        # drop silicon facets
silicon accel <agent> <work>  # schedule compute burst
silicon declare nvidia …      # declare a die when probe is unavailable (labs)
```

## Policy

- `private` vessels cannot receive telemetry, but **can** bind local silicon.
- Focus shield does not steal GPU mid-burst; accel intents are urgency-aware.
- Binding requires vessel facet `silicon.bind` (developer policy includes it).

## Host probe vs lab declare

On machines without NVIDIA/AMD present, `silicon probe` reports empty vendor sets and developers may `silicon declare` for bringing up agents against a target topology (e.g. dual RTX + Epyc).
