# COMP0016 Project Specification

## Document Details

| Field | Value |
| ------- | ------- |
| Version number and date of this document | v1.0.0 25/10/2026 |
| Team No. | 12 |
| Working Project Title | Connected-Care-WiFi-7 |
| Project GitHub Repo URL | <https://github.com/danielharrington06/Connected-Care-WiFi-7> |
| Academic Supervisor Name | Graham Roberts |
| IXN Contact Lead Name | Dean Mohamedally |
| TA Name | Doga Yilmaz |

## Team

| Name | Email | Role |
| ------ | ------- | ------ |
| Daniel Harrington | <daniel.harrington.25@ucl.ac.uk> | Team Lead |
| Matej Pechoucek | <matej.pechoucek.25@ucl.ac.uk> | TBD |
| Magizh Umashankar | <magizh.umashankar.25@ucl.ac.uk> | TBD |
| Daniel Wheeler | <daniel.wheeler.25@ucl.ac.uk> | TBD |
| Ben Dalton | <benjamin.dalton.25@ucl.ac.uk> | TBD |
| Mariha Subhan | <mariha.subhan.24@ucl.ac.uk> | Final Year Student Collaborator |

## Partner Organisation

| Field | Value |
| ----- | ----- |
| Partner Organisation Name | Intel Capital |
| Address of Partner Organisation | N/A |

### Overview of Partner Organisation

Add content here

### Partner Contacts

| Contact Name | Job Title | Email | Phone Number |
| ------------ | --------- | ----- | ------------ |
| Phillippa Chick | N/A | <phillippa.chick@intel.com> | N/A |
| Christian A. Buerger | N/A | <christian.a.buerger@intel.com> | N/A |

### Communication and Meetings

> How often the team meets the partner, through which channel (e.g., Teams,
> email, Slack), and who the main contact is on each side.

Add content here

## Project Description

> Summarise the project background and the problem(s) to be solved. Please
> list the key outcomes and deliverables that might be expected (e.g., a
> proof-of-concept design and implementation of software).

Connected Care with WiFi 7 is an open-source, privacy-first wireless sensing and
edge-AI research demonstrator engineered to investigate how Intel Wi-Fi 7
technology can detect patient falls, quantify gait instability, and map occupant
motion across care homes and clinical environments. Developed by a 2nd-year UCL
Computer Science IXN team in direct collaboration with a UCL final-year thesis
student, the platform explores the sensory capabilities of next-generation
wireless communications by extracting high-resolution Channel State Information
(CSI) and radio frequency (RF) perturbations across 320 MHz channels and
multi-link operations (MLO). From these wireless signals, the edge runtime
reconstructs real-time 3D volumetric spatial occupancy maps of moving objects
within the RF propagation sphere, eliminating the need for intrusive optical
cameras or wearable panic pendants that vulnerable residents frequently forget
or refuse to wear.

Operating entirely on local edge compute, the system deploys machine learning
classifiers to distinguish benign movements (such as sitting down, lying in bed,
or walking) from high-risk acute fall events. It provides an intuitive spatial
configuration interface enabling care home staff to designate specific physical
spaces, corridors, or bed perimeters as customized alert triggers, dispatching
low-latency network pulses when anomalous trajectories occur. Grounded strictly
in data minimization, university ethics, and GDPR guidelines, the project
conducts rigorous benchmarking on network throughput, detection latency, CSI
availability, and false-positive alert risks using open-access benchmarks,
synthetic waveforms, and controlled lab trials. The outcome is a validated
research demonstrator establishing the technical viability and boundary
limitations of Wi-Fi 7 as a touchless, privacy-preserving ambient healthcare
infrastructure.

Unobserved patient falls in residential care facilities and geriatric wards
represent a major cause of severe morbidity, hospitalization, and healthcare
expenditure. Traditional monitoring systems suffer from severe trade-offs:
optical cameras raise unacceptable privacy concerns in private bedrooms and
bathrooms, while wearable accelerometers and emergency lanyards suffer from low
adherence. By harnessing Intel Wi-Fi 7 wireless sensing, ambient radio waves can
passively image spatial movement and identify rapid drops in physical elevation
without collecting identifiable imagery or demanding wearable compliance.

- **Intel Wi-Fi 7 Sensing Pipeline:** Extract and process Channel State
  Information (CSI) and subcarrier phase/amplitude perturbations using Intel
  Wi-Fi 7 network hardware across wideband (up to 320 MHz) frequencies.
- **3D Volumetric RF Spatial Mapping:** Reconstruct a real-time 3D spatial
  coordinate occupancy map of moving human objects within the RF propagation
  sphere.
- **Machine Learning Fall & Anomaly Classifier:** Train edge ML algorithms to
  detect movement dynamics characteristic of acute falls, discriminating them
  from everyday activities of daily living (ADLs) such as sitting, bending, or
  reclining.
- **Spatial Movement Trigger & Alert Framework:** Allow care home administrators
  to record and configure spatial zones and movement trajectories as
  programmable event triggers, status pulses, and automated nurse-call
  notifications.
- **Network & Sensing Performance Evaluation:** Benchmark detection latency,
  packet availability, classification accuracy, and network QoS to quantify the
  trade-offs of concurrent sensing and data communication over Wi-Fi 7.
- **Ethics, GDPR & Data Minimisation:** Ensure 100% adherence to privacy-first
  design, processing zero personally identifiable information (PII), and
  utilizing controlled demonstrators, public benchmarks, and synthetic datasets.

## Timeline and Milestones

> Key dates, interim deliverables and partner demo points.

- **Phase 1: Wi-Fi 7 CSI Ingestion & Testbed Setup (Oct – Nov 2026), Weeks 1 –
  4:** Configure Intel Wi-Fi 7 testbed with final year student; implement CSI
  extraction driver; benchmark baseline packet availability and noise floors.
- **Phase 2: Signal Filtering & 3D Mapping Core (Nov – Dec 2026), Weeks 5 – 8:**
  Build phase unwrapping and wavelet filters; reconstruct 3D occupancy voxel
  grid; train baseline fall classifier on synthetic and open datasets.
- **Phase 3: Trigger Automation & Feasibility Benchmarking (Jan – Feb 2027),
  Weeks 9 – 12:** Implement user-defined spatial alert triggers; test
  false-positive rejection; evaluate network throughput trade-offs; author
  technical evaluation.
- **Hard Deadline 1: Software Delivery, Friday, 12 March 2027:** Full Software
  Code Delivery: Feature-frozen CSI pipeline, 3D visualizer, fall detection
  model, alert manager, test suites, and documentation.
- **Hard Deadline 2: Website & Video Delivery, Tuesday, 16 March 2027:** Public
  IXN Website & Video: Complete project website, technical architecture
  walkthrough, demonstrator showcase video, and research report.

## Detailed Requirements

> List the requirements (must have, should have) that you will be working on
> and that should be met to make the project a success.

Must Have Requirements (Critical Delivery):

- **FR-01: Wi-Fi 7 CSI Data Capture Pipeline:** Implement a driver and telemetry
  ingestion service interfacing with Intel Wi-Fi 7 hardware to extract raw
  subcarrier Channel State Information (CSI) and amplitude/phase matrices.
- **FR-02: Edge-Based Fall Detection Classifier:** Train and deploy a local
  machine learning classifier to identify acute fall signatures, distinguishing
  rapid vertical descents from controlled reclining or sitting.
- **FR-03: Spatial Trigger & Motion Alert Framework:** Enable caregivers to set
  recorded movements or designated 3D coordinate volumes as custom event
  triggers that dispatch alerts upon detection.
- **FR-04: Local Edge Processing (Zero Cloud Dependency):** Execute all signal
  processing, 3D mapping, and inference locally on an on-premise edge gateway
  without transmitting raw RF or sensor data off-site.
- **FR-05: Privacy-First Synthetic & Controlled Testing:** Conduct all model
  training and validation using public anonymized datasets, synthetic RF
  perturbations, or controlled laboratory simulations with zero PII.
- **FR-06: Feasibility & Performance Evaluation Report:** Deliver a
  comprehensive academic and empirical evaluation analyzing Wi-Fi 7 sensing
  accuracy, latency, packet availability, and concurrent communication
  throughput.
- **FR-07: Multi-Occupant Trajectory Tracking:** Differentiate and track
  separate spatial centroids when multiple moving individuals occupy the same
  room or hallway.
- **FR-8: Caregiver Dashboard:** Deliver an intuitive desktop
  dashboard displaying room occupancy heatmaps, trigger boundaries, and live
  alert telemetry.
- **FR-9: 2D RF Spatial Coordinate Mapping:** Implement a 2D 
  occupancy map which displays moving objects and humans within a 2D space
  and height resembled in sheets of 2D planes.

Should Have Requirements (High Priority):

- **FR-10: 3D RF Spatial Coordinate Mapping:** Generate an interactive,
  real-time 3D visual occupancy map displaying moving objects and human
  centroids within the RF propagation sphere.
- **FR-11: False-Alarm Mitigation Filter:** Implement heuristic temporal
  debouncing and secondary state verification to prevent false alarms from
  dropped objects or pets.
- **FR-12: Complementary Sensor Fusion Hooks:** Provide software ingestion
  interfaces for complementary low-cost sensors (e.g. floor pressure pads, PIR
  motion detectors, radar) to cross-validate Wi-Fi 7 detections.

## Stretch Requirements

> Additional requirements if time permits.

Could Have Requirements:

- **FR-13: Multi-Link Operation (MLO) Band Steering:** Evaluate dynamic band
  switching across 5 GHz and 6 GHz to prioritize sensing sensitivity during high
  network traffic.
- **FR-14: Subtle Respiration & Micro-Movement Sensing:** Experiment with
  subcarrier phase variance to estimate stationary chest movements and nocturnal
  resting patterns when occupants are in bed.
- **FR-15: Smart Facility IoT Actuation:** Dispatch automated IoT triggers to
  illuminate pathway lighting or unlock automated room doors when an occupant
  stands up at night.

## Target Users and Stakeholders

> Who will actually use the system (the partner's staff, customers, the
> public). Requirements, testing and ethics decisions all depend on this.

Add content here

## Out of Scope

> What the project will *not* deliver, to stop scope creep.

Won't Have Requirements:

- **Formal Clinical Diagnostic Validation:** The platform is strictly an
  exploratory engineering prototype and research demonstrator; it does not
  replace professional care, certified medical devices, or clinical diagnosis.
- **Live Uncontrolled Clinical Deployment:** Testing is strictly confined to lab
  testbeds, public benchmarks, and synthetic data; unsupervised deployment in
  live patient care rooms without ethical clearance is prohibited.
- **Proprietary RF Silicon Modification:** The project utilizes commercial
  off-the-shelf Intel Wi-Fi 7 chipsets and standard driver interfaces; custom
  baseband silicon fabrication is out of scope.

## Existing Systems and Integration

> Any existing codebase, APIs, databases or platforms the project must work
> with, and whether or when the team will get access to them.

Add content here

## Constraints and Risks

> Identify any constraints on what can be done (e.g., time, technology, data
> access) and how risk can be mitigated (managed).

Add content here

## Acceptance Criteria

> How the partner will judge whether each requirement has been met. How will
> this be confirmed?

- **Fall Detection Sensitivity & Specificity:** Achieve >= 90% true-positive
  fall detection sensitivity with <= 5% false-positive rate across controlled
  laboratory validation trials.
- **End-to-End Alert Latency:** Fall event occurrence to trigger alert dispatch
  latency <= 250 ms across the local network.
- **3D Spatial Resolution:** Voxel tracking accuracy within <= 35 cm spatial
  error margin for human centroid localization under line-of-sight conditions.
- **Network Throughput Impact:** Sustain <= 8% reduction in concurrent Wi-Fi 7
  data transmission throughput while maintaining high-rate CSI extraction.
- **Privacy & Ethics Compliance:** 100% verified zero PII capture, strictly
  localized edge compute, and formal university ethics checklist fulfillment.

## Software Testing and AI Evaluation

> What are the strategies for testing the software (unit, functional,
> integration, regression)? How will the output from AI be evaluated for
> correctness and accuracy?

- **Network & Sensing Performance Evaluation:** Benchmark detection latency,
  packet availability, classification accuracy, and network QoS to quantify the
  trade-offs of concurrent sensing and data communication over Wi-Fi 7.
- **FR-05: Privacy-First Synthetic & Controlled Testing:** Conduct all model
  training and validation using public anonymized datasets, synthetic RF
  perturbations, or controlled laboratory simulations with zero PII.
- **FR-06: Feasibility & Performance Evaluation Report:** Deliver a
  comprehensive academic and empirical evaluation analyzing Wi-Fi 7 sensing
  accuracy, latency, packet availability, and concurrent communication
  throughput.

## Technical and/or Mentor Support Availability

> Will technical support or a mentor be provided by the project partner? In
> particular, for more specialised domain or development knowledge.

Add content here

## Design and Technical Challenges

> Including mobile, architecture, data analysis, AI/ML, LLMs,
> graphics, VR/AR, security, algorithms, and anything else.

Connected Care with WiFi 7 is engineered as a decoupled, edge-first architecture
comprising five core layers:

1. **Wi-Fi 7 Hardware & CSI Ingestion Layer:** Interfaces with Intel Wi-Fi 7
   wireless network adapters (e.g. Intel BE200) to capture raw high-bandwidth
   CSI subcarrier matrices, Doppler shifts, and time-of-flight phase shifts
   across 2.4, 5, and 6 GHz bands.
2. **Signal Pre-processing & Noise Filtration Core:** Filters ambient multipath
   noise, antenna cross-talk, and static furniture reflections using Hampel
   filtering, discrete wavelet transforms (DWT), and phase unwrapping
   algorithms.
3. **3D RF Spatial Tomography & Occupancy Engine:** Translates processed
   multipath RF disturbances into real-time 3D voxel grids and kinematic
   trajectory bounding boxes representing moving occupants within the room.
4. **Edge ML Fall Recognition & Trajectory Model:** Evaluates kinetic velocity
   vectors and centroid height changes using lightweight temporal convolutional
   networks (TCN) or recurrent neural networks running locally on the edge
   gateway.
5. **Spatial Trigger Configuration & Alert Hub:** Administrative dashboard for
   care staff to define interactive trigger zones (e.g. bed boundary, bathroom
   entrance) and dispatch automated network pulses, nurse-station alerts, or
   MQTT/Webex event payloads.

## Knowledge/Skills Required

> Any particular knowledge or skills that are needed, or need to be learnt?

Add content here

## Devices and Software

> Software, hardware or cloud services that are required for the project. If
> specific hardware is needed will it be provided by the project partner? If
> access to a paid-for cloud service is needed (e.g., Azure, AWS, OpenAI,
> Anthropic, etc.), will it be funded by the project partner?

- **Wireless Standard & Hardware:** IEEE 802.11be (Wi-Fi 7), Intel BE200
  PCIe/M.2 NICs.

## Compute and GPU Resources

> Specific resources needed to support AI use or development. Will they be
> available?

- **FR-04: Local Edge Processing (Zero Cloud Dependency):** Execute all signal
  processing, 3D mapping, and inference locally on an on-premise edge gateway
  without transmitting raw RF or sensor data off-site.

## Data

> Any data required for the project. Is it freely available, and not subject
> to any restrictions such as copyright? Will the data be available on time,
> and does it have any special storage requirements (e.g., due to size or
> restricted accessibility)?

- **FR-05: Privacy-First Synthetic & Controlled Testing:** Conduct all model
  training and validation using public anonymized datasets, synthetic RF
  perturbations, or controlled laboratory simulations with zero PII.

## Ethical Approval and GDPR Compliance

> Does the project involve collecting or using data that can identify
> individuals, working with people to collect data or conduct experiments, or
> does it require secure storage or limited access to data, raise moral issues
> about the work to be undertaken or the use of the results of the work?
>
> If needed, has ethical approval been applied for or granted?

- **Ethics, GDPR & Data Minimisation:** Ensure 100% adherence to privacy-first
  design, processing zero personally identifiable information (PII), and
  utilizing controlled demonstrators, public benchmarks, and synthetic datasets.
- **Privacy & Ethics Compliance:** 100% verified zero PII capture, strictly
  localized edge compute, and formal university ethics checklist fulfillment.

## IP and Licensing

> Who owns the code the team writes, and what licence it will be released
> under (open source, proprietary, partner-owned)?

Add content here

## Code Hosting and Hand-over

> What happens to the GitHub repo, who has access, and what gets handed over
> at the end (code, documentation, deployment).

Add content here

## Any Other Information

Note: All project requirements are subject to discussions and changes can be
made by the signatory clients.

## Change Log

| Version | Date       | Author            | Summary of Changes                                              |
| ------- | ---------- | ----------------- | --------------------------------------------------------------- |
| 1.0.0   | 25/10/2026 | Matej Pechoucek   | Initial Background Information                                  |
| 1.0.1   | 25/10/2026 | Matej Pechoucek   | Extraction of info from docx                                    |
| 1.0.2   | 27/10/2026 | Daniel Harrington | MoSCoW text added to stretch and out of scope requirements      |
| 1.0.3   | 17/11/2026 | Matej Pechoucek   | Moved requirements to different necessity levels                |
| 1.0.4   | 18/11/2026 | Matej Pechoucek   | Re-numbered requirements and added 2D visualization requirement |
