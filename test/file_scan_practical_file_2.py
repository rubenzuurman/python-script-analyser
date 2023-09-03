import math

from player import Player

class SimplePlayerRotateShoot(Player):
    
    def __init__(self):
        super().__init__()
    
    def update(self, input_array):
        # Get ns/ew from rotation.
        ns = math.cos(self.rotation)
        ew = math.sin(self.rotation)
        
        # Extract opponent sensors from input array.
        opponent_sensors = [input_array[i] for i in range(2, 3 * self.num_rays, 3)]
        
        # Detect opponents.
        left_detected = True in [v > 0 for v in opponent_sensors[:(len(opponent_sensors) - 1) // 2]]
        center_detected = opponent_sensors[(len(opponent_sensors) - 1) // 2] > 0
        right_detected = True in [v > 0 for v in opponent_sensors[(len(opponent_sensors) - 1) // 2 + 1:]]
        
        angular_velocity = 0
        velocity = 0
        activate_weapon = -1
        
        # Check if any of the 'left' rays detect a player.
        if left_detected:
            velocity = 0.05
            angular_velocity = -0.50
            activate_weapon = -1.0
        
        # Check if any of the 'right' rays detect a player.
        if right_detected:
            velocity = 0.05
            angular_velocity = 0.50
            activate_weapon = -1.0
        
        # Check if middle ray detects player.
        if center_detected:
            velocity = 0.20
            angular_velocity = 0
            if left_detected:
                angular_velocity += -0.20
            if right_detected:
                angular_velocity += +0.20
            activate_weapon = float(opponent_sensors[(len(opponent_sensors) - 1) // 2] > 0)
        
        if not left_detected and not right_detected and not center_detected:
            angular_velocity = 0.80
        
        # Return velocity, angular velocity, and activate weapon output 
        # (probably (at least currently) active when value is greater than 0).
        return velocity, angular_velocity, activate_weapon
