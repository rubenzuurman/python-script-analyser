import math
import pickle

import pygame

def render_text(display, text, position, font, color=(255, 255, 255)):
    text_surface = font.render(text, False, color)
    display.blit(text_surface, position)

def load_tests():
    with open("tests.pickle", "rb") as file:
        return pickle.load(file)

def main():
    pygame.init()
    pygame.font.init()
    font = pygame.font.SysFont("Courier New", 16)
    
    # Create window.
    window_dimensions = (1200, 1000)
    display = pygame.display.set_mode(window_dimensions, pygame.RESIZABLE)
    
    # Load tests.
    tests = load_tests()
    test_number = 0
    
    # Start render loop.
    fps = 60
    clock = pygame.time.Clock()
    running = True
    while running:
        for event in pygame.event.get():
            if event.type == pygame.QUIT:
                running = False
            
            if event.type == pygame.KEYDOWN:
                if event.key == pygame.K_ESCAPE:
                    running = False
                
                if event.key == pygame.K_SPACE:
                    test_number += 1
                    test_number = test_number % len(tests)
            
            if event.type == pygame.VIDEORESIZE:
                window_dimensions = (event.w, event.h)
                display = pygame.display.set_mode(window_dimensions, pygame.RESIZABLE)
        
        display.fill((0, 0, 0))
        
        test = tests[test_number]
        
        # Render test number and expected result
        render_text(display, f"Test #{test_number+1}", (10, 5), font)
        render_text(display, f"Line 1: {test['l1']}", (10, 25), font)
        render_text(display, f"Line 2: {test['l2']}", (10, 45), font)
        render_text(display, f"Expected result: {test['out']}", (10, 65), font)
        
        # Render grid.
        for gridx in range(-25, 26, 5):
            color = (50, 50, 50)
            if gridx == 0:
                color = (200, 200, 200)
            pygame.draw.line(display, color, \
                convert_test_space_to_screen_space((gridx, -30), window_dimensions), \
                convert_test_space_to_screen_space((gridx, 30), window_dimensions))
        for gridy in range(-25, 26, 5):
            color = (50, 50, 50)
            if gridy == 0:
                color = (200, 200, 200)
            pygame.draw.line(display, color, \
                convert_test_space_to_screen_space((-30, gridy), window_dimensions), \
                convert_test_space_to_screen_space((30, gridy), window_dimensions))
        
        # Render line 1
        l1p1 = convert_test_space_to_screen_space(test["l1"][0], window_dimensions)
        l1p2 = convert_test_space_to_screen_space((test["l1"][0][0] + 50 * math.cos(test["l1"][1]), test["l1"][0][1] + 50 * math.sin(test["l1"][1])), window_dimensions)
        pygame.draw.line(display, (5, 116, 207), l1p1, l1p2)
        pygame.draw.circle(display, (5, 116, 207), \
            convert_test_space_to_screen_space(test["l1"][0], window_dimensions), 5)
        
        # Render line 2
        l2p1 = convert_test_space_to_screen_space(test["l2"][0], window_dimensions)
        l2p2 = convert_test_space_to_screen_space(test["l2"][1], window_dimensions)
        pygame.draw.line(display, (189, 129, 69), l2p1, l2p2)
        
        pygame.display.flip()
        clock.tick(fps)
    
    pygame.quit()

def convert_test_space_to_screen_space(p, window_dimensions):
    # x -30 to 30
    # y -30 to 30
    x = p[0]
    y = -p[1]
    x = (x / 30) * (window_dimensions[0] / 2) + (window_dimensions[0] / 2)
    y = (y / 30) * (window_dimensions[1] / 2) + (window_dimensions[1] / 2)
    return x, y

if __name__ == "__main__":
    main()