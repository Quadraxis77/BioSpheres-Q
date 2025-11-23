# Genome File Format

BioSpheres-Q genomes can be saved and loaded as human-readable JSON files for easy sharing and version control.

## Usage

### Saving a Genome
1. Open the Genome Editor in Preview mode
2. Edit your genome as desired
3. Enter a name for your genome in the "Genome Name" field
4. Click the "Save Genome" button
5. Choose a location and filename (the `.json` extension will be added automatically)

### Loading a Genome
1. Open the Genome Editor in Preview mode
2. Click the "Load Genome" button
3. Select a genome JSON file
4. The genome will be loaded and the editor will update to show all modes and settings

## File Format

Genomes are stored as JSON files with the following structure:

```json
{
  "name": "Genome Name",
  "initial_mode": 0,
  "initial_orientation": {
    "x": 0.0,
    "y": 0.0,
    "z": 0.0,
    "w": 1.0
  },
  "modes": [
    {
      "name": "Mode Name",
      "default_name": "Default Mode Name",
      "color": {
        "x": 1.0,
        "y": 1.0,
        "z": 1.0
      },
      "cell_type": 0,
      "parent_make_adhesion": false,
      "split_mass": 1.0,
      "split_interval": 5.0,
      "parent_split_direction": {
        "x": 0.0,
        "y": 0.0
      },
      "max_adhesions": 20,
      "enable_parent_angle_snapping": true,
      "child_a": {
        "mode_number": 0,
        "orientation": {
          "x": 0.0,
          "y": 0.0,
          "z": 0.0,
          "w": 1.0
        },
        "keep_adhesion": true,
        "enable_angle_snapping": true
      },
      "child_b": {
        "mode_number": 0,
        "orientation": {
          "x": 0.0,
          "y": 0.0,
          "z": 0.0,
          "w": 1.0
        },
        "keep_adhesion": true,
        "enable_angle_snapping": true
      },
      "adhesion_settings": {
        "can_break": true,
        "break_force": 10.0,
        "rest_length": 1.0,
        "linear_spring_stiffness": 150.0,
        "linear_spring_damping": 5.0,
        "orientation_spring_stiffness": 50.0,
        "orientation_spring_damping": 5.0,
        "max_angular_deviation": 0.0,
        "twist_constraint_stiffness": 2.0,
        "twist_constraint_damping": 0.5,
        "enable_twist_constraint": false
      }
    }
  ]
}
```

## Field Descriptions

### Genome-level Fields
- `name`: Display name of the genome
- `initial_mode`: Index of the mode that new organisms start in
- `initial_orientation`: Initial quaternion orientation (x, y, z, w components)
- `modes`: Array of mode definitions

### Mode Fields
- `name`: Current display name of the mode
- `default_name`: Default/fallback name for the mode
- `color`: RGB color (values from 0.0 to 1.0)
- `cell_type`: Cell type identifier (currently only 0 = "Test")
- `parent_make_adhesion`: Whether parent cells create adhesions when dividing
- `split_mass`: Mass allocated to each child cell during division
- `split_interval`: Time (in seconds) between divisions
- `parent_split_direction`: Split direction as pitch (x) and yaw (y) in degrees
- `max_adhesions`: Maximum number of adhesion connections allowed
- `enable_parent_angle_snapping`: Whether to snap angles to 11.25° grid

### Child Settings (child_a and child_b)
- `mode_number`: Index of the mode that this child cell will adopt
- `orientation`: Quaternion orientation relative to parent (x, y, z, w)
- `keep_adhesion`: Whether child inherits parent's adhesions
- `enable_angle_snapping`: Whether to snap child orientation to grid

### Adhesion Settings
Physics parameters for cell-cell connections:
- `can_break`: Whether adhesions can be broken by force
- `break_force`: Force threshold for breaking adhesions
- `rest_length`: Equilibrium distance for adhesion spring
- `linear_spring_stiffness`: Stiffness of linear spring (higher = stiffer)
- `linear_spring_damping`: Damping of linear oscillations
- `orientation_spring_stiffness`: Stiffness of rotational alignment
- `orientation_spring_damping`: Damping of rotational oscillations
- `max_angular_deviation`: Maximum angular deviation allowed (degrees)
- `twist_constraint_stiffness`: Resistance to twisting motion
- `twist_constraint_damping`: Damping of twist oscillations
- `enable_twist_constraint`: Whether twist constraints are active

## Example

See `example_genome.json` in the project root for a complete example with two modes demonstrating a stem cell that differentiates into a specialized cell type.

## Tips for Sharing Genomes

1. Use descriptive names for your genomes and modes
2. Include comments in mode names to describe their function (e.g., "Stem Cell - Divides asymmetrically")
3. Keep colors distinct for easy visualization
4. Test genomes in Preview mode before sharing
5. Version control your genome files with git for easy collaboration

## Technical Notes

- Quaternions are stored in (x, y, z, w) format
- Angles are in degrees for human readability
- Colors use normalized RGB values (0.0-1.0)
- The file format uses pretty-printed JSON with 2-space indentation for readability
